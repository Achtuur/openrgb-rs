use std::mem::size_of;

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
        buf.pop();
        String::from_utf8(buf).map_err(|e| {
            OpenRgbError::ProtocolError(format!("Failed decoding string as UTF-8: {}", e))
        })
    }
}

#[doc(hidden)]
pub struct RawString<'a>(pub &'a str);

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
