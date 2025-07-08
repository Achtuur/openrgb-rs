use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};

use crate::protocol::MAGIC;
use crate::{OpenRgbError, OpenRgbResult};

use super::{DEFAULT_PROTOCOL, PacketId, TryFromStream, Writable};

pub trait ProtocolStream {
    fn protocol_version(&self) -> u32;
    fn set_protocol_version(&mut self, version: u32);
}

pub trait ReadableStream: ProtocolStream + AsyncReadExt + Sized + Send + Sync + Unpin {
    async fn read_value<T: TryFromStream>(&mut self) -> OpenRgbResult<T> {
        T::try_read(self).await
    }

    async fn read_header(
        &mut self,
        expected_device_id: u32,
        expected_packet_id: PacketId,
    ) -> OpenRgbResult<usize> {
        tracing::trace!("Reading {:?} packet...", expected_packet_id);

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

pub trait WritableStream: ProtocolStream + AsyncWriteExt + Sized + Send + Sync + Unpin {
    /// Writes a value of type T to the stream, returns the number of bytes written.
    async fn write_value<T: Writable>(&mut self, value: &T) -> OpenRgbResult<usize> {
        let n = T::try_write(value, self).await?;
        assert!(n == value.size(), "size mismatch: expected {}, got {}", value.size(), n);
        Ok(n)
    }

    async fn write_header(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
        data_len: usize,
    ) -> OpenRgbResult<()> {
        tracing::trace!("Sending {:?} packet of {} bytes...", packet_id, data_len);
        self.write_all(&MAGIC).await?;
        self.write_value(&device_id).await?;
        self.write_value(&packet_id).await?;
        self.write_value(&data_len).await?;
        Ok(())
    }

    async fn write_packet<I: Writable>(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
        data: &I,
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
            tracing::trace!("header: {0:?}", &buf[..16]);
            tracing::trace!("packet: {0:?}", &buf[16..]);
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
    // fn protocol_version(&self) -> u32 {
    //     <Self as ProtocolStream>::protocol_version(self)
    // }

    async fn request<I: Writable, O: TryFromStream>(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
        data: &I,
    ) -> OpenRgbResult<O> {
        self.write_packet(device_id, packet_id, data).await?;
        self.read_packet(device_id, packet_id).await
    }
}

impl<S> OpenRgbStream for S where S: ReadableStream + WritableStream {}

pub(crate) struct ProtocolTcpStream {
    stream: TcpStream,
    protocol_version: u32,
}

impl ProtocolTcpStream {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let protocol_version = DEFAULT_PROTOCOL;
        Ok(Self {
            stream,
            protocol_version,
        })
    }

    pub fn peer_addr(&self) -> Result<std::net::SocketAddr, std::io::Error> {
        self.stream.peer_addr()
    }
}

impl ProtocolStream for ProtocolTcpStream {
    fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    fn set_protocol_version(&mut self, version: u32) {
        self.protocol_version = version;
    }
}

impl AsyncRead for ProtocolTcpStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let pin = Pin::new(&mut self.stream);
        AsyncRead::poll_read(pin, cx, buf)
    }
}

impl AsyncWrite for ProtocolTcpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let pin = Pin::new(&mut self.get_mut().stream);
        AsyncWrite::poll_write(pin, cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let pin = Pin::new(&mut self.get_mut().stream);
        AsyncWrite::poll_flush(pin, cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let pin = Pin::new(&mut self.get_mut().stream);
        AsyncWrite::poll_shutdown(pin, cx)
    }
}

impl ReadableStream for ProtocolTcpStream {}
impl WritableStream for ProtocolTcpStream {}

#[cfg(debug_assertions)]
impl ProtocolStream for Vec<u8> {
    fn protocol_version(&self) -> u32 {
        DEFAULT_PROTOCOL
    }

    fn set_protocol_version(&mut self, _version: u32) {}
}

#[cfg(debug_assertions)]
impl WritableStream for Vec<u8> {}
