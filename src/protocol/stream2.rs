use std::{io::{Cursor, Read, Write}, pin::Pin};

use tokio::{io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt}, net::{TcpStream, ToSocketAddrs}};

use crate::{OpenRgbError, OpenRgbResult, PacketId, DEFAULT_PROTOCOL};


/// Utility struct to write packets.
/// Some packets need to be prepended by their length.
/// This struct serializes the contents and prepends the length to the buffer.
pub(crate) struct OpenRgbPacket<T: SerToBuf> {
    pub contents: T,
}

impl<T: SerToBuf> OpenRgbPacket<T> {
    pub fn new(contents: T) -> OpenRgbPacket<T> {
        Self { contents }
    }
}

impl<T: SerToBuf> SerToBuf for OpenRgbPacket<T> {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        let mut inner_buf = WriteMessage::new(buf.protocol_version());
        self.contents.serialize(&mut inner_buf)?;
        let len = inner_buf.len() + size_of::<u32>(); // + u32 to account for the length field itself
        buf.write_u32(len as u32);
        buf.extend_from_slice(inner_buf.bytes());
        Ok(())
    }
}

pub(crate) struct OpenRgbMessageHeader {
    packet_id: PacketId,
    device_id: u32,
    packet_size: u32,
}

impl OpenRgbMessageHeader {
    pub(crate) const MAGIC: [u8; 4] = *b"ORGB";

    async fn read(stream: &mut TcpStream) -> OpenRgbResult<Self> {
        // header is always 16 bytes long
        let mut buf = [0u8; 16];
        stream.read_exact(&mut buf).await?;
        let mut recv = ReceivedMessage::new(&buf, 0); // header is constant across protocol versions
        tracing::trace!("Read header: {}", recv);
        let magic = recv.read_value::<[u8; 4]>()?;
        if magic != Self::MAGIC {
            return Err(OpenRgbError::ProtocolError(format!(
                "expected OpenRGB magic value, got {:?}",
                magic
            )));
        }

        let device_id = recv.read_u32()?;
        let packet_id = recv.read_value::<PacketId>()?;
        let packet_size = recv.read_u32()?;
        Ok(Self {device_id, packet_id, packet_size,})
    }

    async fn write(&self, stream: &mut TcpStream) -> OpenRgbResult<()> {
        let mut buf = WriteMessage::with_capacity(0, 16);
        buf.extend_from_slice(&Self::MAGIC);
        buf.write_u32(self.device_id);
        buf.write_value(&self.packet_id)?;
        buf.write_u32(self.packet_size);
        stream.write_all(buf.bytes()).await?;
        Ok(())
    }
}

pub(crate) struct ReceivedMessage<'a> {
    protocol_version: u32,
    buf: &'a [u8],
    idx: usize,
}

impl std::fmt::Display for ReceivedMessage<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Received (protocol: {}, offset: {}): {:?})",
               self.protocol_version, self.idx, self.available_buf())
    }
}

impl<'a> ReceivedMessage<'a> {
    pub fn new(buf: &'a [u8], protocol_version: u32) -> Self {
        Self {
            protocol_version,
            buf,
            idx: 0,
        }
    }

    pub fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    fn available_buf(&self) -> &[u8] {
        &self.buf[self.idx..]
    }

    #[inline]
    pub fn read_u8(&mut self) -> OpenRgbResult<u8> {
        if self.available_buf().is_empty() {
            return Err(OpenRgbError::ProtocolError("Not enough bytes to read u8".to_string()));
        }
        let byte = self.buf[self.idx];
        self.idx += 1;
        Ok(byte)
    }

    pub fn read_u16(&mut self) -> OpenRgbResult<u16> {
        let b = self.available_buf();
        if b.len() < 2 {
            return Err(OpenRgbError::ProtocolError("Not enough bytes to read u16".to_string()));
        }
        let value = u16::from_le_bytes([b[0], b[1]]);
        self.idx += 2;
        Ok(value)
    }

    pub fn read_u32(&mut self) -> OpenRgbResult<u32> {
        let b = self.available_buf();
        if b.len() < 4 {
            return Err(OpenRgbError::ProtocolError("Not enough bytes to read u16".to_string()));
        }
        let value = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        self.idx += 4;
        Ok(value)
    }

    pub fn read_value<T: DeserFromBuf>(&mut self) -> OpenRgbResult<T> {
        T::deserialize(self)
    }

    /// Reads the next `n` values as type `T` from the buffer.
    ///
    /// If there's a `[len, [..data]]` format, use `read_value::<Vec<T>>()` instead.
    pub fn read_n_values<T: DeserFromBuf>(&mut self, n: usize) -> OpenRgbResult<Vec<T>> {
        let mut values = Vec::with_capacity(n);
        for _ in 0..n {
            values.push(T::deserialize(self)?);
        }
        Ok(values)
    }
}

impl std::io::Read for ReceivedMessage<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let available = &self.buf[self.idx..];
        let len = buf.len().min(available.len());
        buf[..len].copy_from_slice(&available[..len]);
        self.idx += len;
        Ok(len)
    }
}

pub struct WriteMessage {
    protocol_version: u32,
    buf: Vec<u8>,
}

impl std::fmt::Display for WriteMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WriteMessage (protocol: {}, len: {}): {:?}",
               self.protocol_version, self.buf.len(), &self.buf[..])
    }
}

impl WriteMessage {
    pub fn new(protocol_version: u32) -> Self {
        Self::with_capacity(protocol_version, 8)
    }

    pub fn with_capacity(protocol_version: u32, capacity: usize) -> Self {
        Self {
            protocol_version,
            buf: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn bytes(&self) -> &[u8] {
        &self.buf
    }

    pub fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    pub fn write_u8(&mut self, value: u8) {
        self.buf.push(value);
    }

    pub fn write_u16(&mut self, value: u16) {
        let _ = self.write(&value.to_le_bytes());
    }

    pub fn write_u32(&mut self, value: u32) {
        let _ = self.write(&value.to_le_bytes());
    }

    pub fn write_value<T: SerToBuf>(&mut self, value: &T) -> OpenRgbResult<()> {
        value.serialize(self)
    }

    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.buf.extend_from_slice(slice);
    }

    pub fn push_value<T: SerToBuf>(&mut self, value: &T) -> OpenRgbResult<&mut Self> {
        self.write_value(value)?;
        Ok(self)
    }
}

impl std::io::Write for WriteMessage {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}



/// Deserialize an object from a byte buffer.
pub trait DeserFromBuf {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self>
    where
        Self: Sized;
}

/// Serialize an object to a byte buffer.
pub trait SerToBuf {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()>;
}

impl<T: SerToBuf> SerToBuf for &T {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        (*self).serialize(buf)
    }
}


pub struct Stream2 {
    stream: TcpStream,
    protocol_version: u32
}

impl Stream2 {
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

    pub fn protocol_version(&self) -> u32 {
        DEFAULT_PROTOCOL
    }

    pub fn set_protocol_version(&mut self, version: u32) {
        self.protocol_version = version;
    }

    pub async fn request<I: SerToBuf, O: DeserFromBuf>(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
        data: &I,
    ) -> OpenRgbResult<O> {
        self.write_packet(device_id, packet_id, data).await?;
        self.read_packet(device_id, packet_id).await
    }

    pub async fn read_packet<T: DeserFromBuf>(&mut self, device_id: u32, packet_id: PacketId,) -> OpenRgbResult<T> {
        // the header tells us exactly how long the packet is, so we might as well read it all at once
        let header = self.read_header(device_id, packet_id).await?;
        let mut buf = vec![0u8; header.packet_size as usize];
        self.stream.read_exact(&mut buf).await?;
        let mut recv = ReceivedMessage::new(&buf, self.protocol_version());
        tracing::trace!("Read packet: {}", recv);
        T::deserialize(&mut recv)
    }

    pub async fn write_packet<T: SerToBuf>(&mut self, device_id: u32, packet_id: PacketId, data: &T) -> OpenRgbResult<()> {
        // let mut buf = Vec::with_capacity(8);
        let mut buf = WriteMessage::new(self.protocol_version());
        data.serialize(&mut buf)?;
        let packet_size = buf.len() as u32;
        let header = OpenRgbMessageHeader {
            packet_id, device_id, packet_size,
        };
        header.write(&mut self.stream).await?;

        tracing::debug!("Writing packet: {}", buf);
        self.stream.write_all(buf.bytes()).await?;
        Ok(())
    }

    async fn read_header(&mut self, device_id: u32, packet_id: PacketId) -> OpenRgbResult<OpenRgbMessageHeader> {
        let header = OpenRgbMessageHeader::read(&mut self.stream).await?;
        if header.packet_id != packet_id {
            return Err(OpenRgbError::ProtocolError(format!(
                "Unexpected packet ID: expected {:?}, got {:?}",
                packet_id, header.packet_id
            )));
        }
        if header.device_id != device_id {
            return Err(OpenRgbError::ProtocolError(format!(
                "Unexpected device ID: expected {}, got {}",
                device_id, header.device_id
            )));
        }
        Ok(header)
    }
}


impl AsyncRead for Stream2 {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let pin = Pin::new(&mut self.stream);
        AsyncRead::poll_read(pin, cx, buf)
    }
}

impl AsyncWrite for Stream2 {
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