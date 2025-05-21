use std::mem::size_of;

use crate::protocol::{ReadableStream, TryFromStream, Writable, WritableStream};
use crate::{OpenRgbError, OpenRgbResult};

// FIXME buggy for non ASCII strings

impl Writable for String {
    fn size(&self) -> usize {
        size_of::<u16>() // string is preceded by its length
        + self.len()
        + 1 // null terminator
    }

    async fn try_write(self, stream: &mut impl WritableStream) -> OpenRgbResult<()> {
        stream.write_value((self.len() + 1) as u16).await?;
        stream.write_value(RawString(self)).await
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
pub struct RawString(pub String);

impl Writable for RawString {
    fn size(&self) -> usize {
        self.0.len() + 1
    }

    async fn try_write(self, stream: &mut impl WritableStream) -> OpenRgbResult<()> {
        stream
            .write_all(format!("{}\0", self.0).as_bytes())
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::protocol::data::RawString;
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

        stream.write_value("test".to_string()).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_write_raw_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(b"test\0").build();

        stream.write_value(RawString("test".to_string())).await?;

        Ok(())
    }
}
