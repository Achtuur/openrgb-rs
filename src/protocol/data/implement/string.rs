use std::io::Read;
use std::mem::size_of;

use crate::protocol::stream2::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};
use crate::protocol::{ReadableStream, TryFromStream, Writable, WritableStream};
use crate::{OpenRgbError, OpenRgbResult};

impl Writable for &str {
    fn size(&self) -> usize {
        size_of::<u16>() // string is preceded by its length
        + self.len()
        + 1 // null terminator
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        let padded_len = (self.len() + 1) as u16;
        let mut n = 0;
        n += stream.write_value(&padded_len).await?;
        n += stream.write_value(&RawString(self)).await?;
        Ok(n)
    }
}

// FIXME buggy for non ASCII strings

impl Writable for String {
    fn size(&self) -> usize {
        self.as_str().size()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        self.as_str().try_write(stream).await
    }
}

impl TryFromStream for String {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        let mut buf = vec![Default::default(); stream.read_value::<u16>().await? as usize];
        stream.read_exact(&mut buf).await?;
        buf.pop(); // null byte?
        String::from_utf8(buf).map_err(|e| {
            OpenRgbError::ProtocolError(format!("Failed decoding string as UTF-8: {}", e))
        })
    }
}

impl DeserFromBuf for String {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self>
    where
        Self: Sized {
        let len = buf.read_u16()? as usize;
        let mut bytes = vec![0u8; len];
        buf.read_exact(&mut bytes)?;
        bytes.pop(); // null byte?
        String::from_utf8(bytes).map_err(|e| {
            OpenRgbError::ProtocolError(format!("Failed decoding string as UTF-8: {}", e))
        })
    }
}


impl SerToBuf for String {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        self.as_str().serialize(buf)
    }
}

impl SerToBuf for &str {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u16(self.len() as u16 + 1); // +1 for null terminator
        buf.write_value(&RawString(self))?;
        Ok(())
    }
}


/// A raw string that does not include the length in its serialized form.
///
/// If the length is needed, serialize a `&str` or `String` instead.
#[doc(hidden)]
pub struct RawString<'a>(pub &'a str);

impl SerToBuf for RawString<'_> {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.extend_from_slice(self.0.as_bytes());
        buf.write_u8(b'\0');
        Ok(())
    }
}


impl Writable for RawString<'_> {
    fn size(&self) -> usize {
        self.0.len() + 1 // null terminator
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        stream.write_all(self.0.as_bytes()).await?;
        stream.write_all(b"\0").await?;
        Ok(self.0.len() + 1)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::protocol::data::implement::string::RawString;
    use crate::protocol::tests::setup;
    use crate::protocol::{ReadableStream, WritableStream};

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&5_u16.to_le_bytes())
            .read(b"test\0")
            .build();

        assert_eq!(stream.read_value::<String>().await?, "test".to_string());

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(&5_u16.to_le_bytes())
            .write(b"test\0")
            .build();

        stream.write_value(&"test").await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_write_raw_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(b"test\0").build();

        stream.write_value(&RawString("test")).await?;

        Ok(())
    }
}
