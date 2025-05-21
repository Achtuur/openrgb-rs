use log::debug;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::protocol::MAGIC;
use crate::{OpenRgbError, OpenRgbResult};

use super::{PacketId, TryFromStream, Writable};

pub trait ReadableStream: AsyncReadExt + Sized + Send + Sync + Unpin {
    async fn read_value<T: TryFromStream>(&mut self) -> OpenRgbResult<T> {
        T::try_read(self).await
    }

    async fn read_header(
        &mut self,
        expected_device_id: u32,
        expected_packet_id: PacketId,
    ) -> OpenRgbResult<usize> {
        debug!("Reading {:?} packet...", expected_packet_id);

        for c in MAGIC {
            if self.read_u8().await? != c {
                return Err(OpenRgbError::ProtocolError(format!(
                    "expected OpenRGB magic value, got \"{}\"",
                    c
                )));
            }
        }

        let device_id = self.read_value::<u32>().await?;
        if device_id != expected_device_id {
            return Err(OpenRgbError::ProtocolError(format!(
                "expected device ID {}, got {}",
                expected_device_id, device_id
            )));
        }

        let packet_id = self.read_value::<PacketId>().await?;
        if packet_id != expected_packet_id {
            return Err(OpenRgbError::ProtocolError(format!(
                "expected packet ID {:?}, got {:?}",
                expected_packet_id, packet_id
            )));
        }

        self.read_value::<u32>().await?.try_into().map_err(|e| {
            OpenRgbError::ProtocolError(format!("received invalid data length: {}", e))
        })
    }

    async fn read_packet<O: TryFromStream>(
        &mut self,
        expected_device_id: u32,
        expected_packet_id: PacketId,
    ) -> OpenRgbResult<O> {
        self.read_header(expected_device_id, expected_packet_id)
            .await?;
        // TODO check header length vs actual read length
        self.read_value().await
    }
}

pub trait WritableStream: AsyncWriteExt + Sized + Send + Sync + Unpin {
    async fn write_value<T: Writable>(&mut self, value: T) -> OpenRgbResult<()> {
        T::try_write(value, self).await
    }

    async fn write_header(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
        data_len: usize,
    ) -> OpenRgbResult<()> {
        debug!("Sending {:?} packet of {} bytes...", packet_id, data_len);
        self.write_all(&MAGIC).await?;
        self.write_value(device_id).await?;
        self.write_value(packet_id).await?;
        self.write_value(data_len).await?;
        Ok(())
    }

    async fn write_packet<I: Writable>(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
        data: I,
    ) -> OpenRgbResult<()> {
        let size = data.size();

        // in debug builds, use intermediate buffer to ease debugging with Wireshark (see #3)
        #[cfg(debug_assertions)]
        {
            let mut buf: Vec<u8> = Vec::with_capacity(
                4 /* magic */ + 4 /* device id */ + 4 /* packet id */ + 4 /* len */ + size, /* payload size*/
            );
            buf.write_header(device_id, packet_id, size).await?;
            buf.write_value(data).await?;
            self.write_all(&buf).await?;
        }

        // in release builds, write directly
        #[cfg(not(debug_assertions))]
        {
            self.write_header(device_id, packet_id, size).await?;
            self.write_value(data).await?;
        }

        Ok(())
    }
}

pub trait OpenRgbStream: ReadableStream + WritableStream {
    async fn request<I: Writable, O: TryFromStream>(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
        data: I,
    ) -> OpenRgbResult<O> {
        self.write_packet(device_id, packet_id, data).await?;
        self.read_packet(device_id, packet_id).await
    }
}

impl ReadableStream for TcpStream {}

impl WritableStream for TcpStream {}

impl OpenRgbStream for TcpStream {}

#[cfg(debug_assertions)]
impl WritableStream for Vec<u8> {}
