use std::mem::size_of;

use crate::data::{TryFromStream, Writable};
use crate::protocol::{ReadableStream, WritableStream};
use crate::{OpenRgbError, OpenRgbResult};
use crate::OpenRgbError::ProtocolError;

impl Writable for () {
    fn size(&self, _protocol: u32) -> usize {
        0
    }

    async fn try_write(
        self,
        _stream: &mut impl WritableStream,
        _protocol: u32,
    ) -> OpenRgbResult<()> {
        Ok(())
    }
}

impl TryFromStream for () {
    async fn try_read(
        _stream: &mut impl ReadableStream,
        _protocol: u32,
    ) -> Result<Self, OpenRgbError> {
        Ok(())
    }
}

impl Writable for u8 {
    fn size(&self, _protocol: u32) -> usize {
        size_of::<u8>()
    }

    async fn try_write(
        self,
        stream: &mut impl WritableStream,
        _protocol: u32,
    ) -> OpenRgbResult<()> {
        stream.write_u8(self).await.map_err(Into::into)
    }
}

impl TryFromStream for u8 {
    async fn try_read(
        stream: &mut impl ReadableStream,
        _protocol: u32,
    ) -> Result<Self, OpenRgbError> {
        stream.read_u8().await.map_err(Into::into)
    }
}

impl Writable for u16 {
    fn size(&self, _protocol: u32) -> usize {
        size_of::<u16>()
    }

    async fn try_write(
        self,
        stream: &mut impl WritableStream,
        _protocol: u32,
    ) -> OpenRgbResult<()> {
        stream.write_u16_le(self).await.map_err(Into::into)
    }
}

impl TryFromStream for u16 {
    async fn try_read(
        stream: &mut impl ReadableStream,
        _protocol: u32,
    ) -> Result<Self, OpenRgbError> {
        stream.read_u16_le().await.map_err(Into::into)
    }
}

impl Writable for u32 {
    fn size(&self, _protocol: u32) -> usize {
        size_of::<u32>()
    }

    async fn try_write(
        self,
        stream: &mut impl WritableStream,
        _protocol: u32,
    ) -> OpenRgbResult<()> {
        stream.write_u32_le(self).await.map_err(Into::into)
    }
}

impl TryFromStream for u32 {
    async fn try_read(
        stream: &mut impl ReadableStream,
        _protocol: u32,
    ) -> Result<Self, OpenRgbError> {
        stream.read_u32_le().await.map_err(Into::into)
    }
}

impl Writable for i32 {
    fn size(&self, _protocol: u32) -> usize {
        size_of::<i32>()
    }

    async fn try_write(
        self,
        stream: &mut impl WritableStream,
        _protocol: u32,
    ) -> OpenRgbResult<()> {
        stream.write_i32_le(self).await.map_err(Into::into)
    }
}

impl TryFromStream for i32 {
    async fn try_read(
        stream: &mut impl ReadableStream,
        _protocol: u32,
    ) -> Result<Self, OpenRgbError> {
        stream.read_i32_le().await.map_err(Into::into)
    }
}

impl Writable for usize {
    fn size(&self, _protocol: u32) -> usize {
        size_of::<u32>()
    }

    async fn try_write(
        self,
        stream: &mut impl WritableStream,
        protocol: u32,
    ) -> OpenRgbResult<()> {
        stream
            .write_value(
                u32::try_from(self).map_err(|e| {
                    ProtocolError(format!(
                        "Data size is too large to encode: {} ({})",
                        self, e
                    ))
                })?,
                protocol,
            )
            .await
    }
}

impl TryFromStream for usize {
    async fn try_read(
        stream: &mut impl ReadableStream,
        protocol: u32,
    ) -> Result<Self, OpenRgbError> {
        stream.read_value::<u32>(protocol).await.map(|s| s as Self)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::protocol::{ReadableStream, WritableStream};
    use crate::tests::setup;
    use crate::DEFAULT_PROTOCOL;

    #[tokio::test]
    async fn test_read_void_001() -> Result<(), Box<dyn Error>> {
        setup()?;
        let stream = Builder::new().build();
        Ok(())
    }

    #[tokio::test]
    async fn test_write_void_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().build();

        stream.write_value((), DEFAULT_PROTOCOL).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_read_u8_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&[37_u8]).build();

        assert_eq!(stream.read_value::<u8>(DEFAULT_PROTOCOL).await?, 37);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_u8_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&[37_u8]).build();

        stream.write_value(37_u8, DEFAULT_PROTOCOL).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_read_u16_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&37_u16.to_le_bytes()).build();

        assert_eq!(stream.read_value::<u16>(DEFAULT_PROTOCOL).await?, 37);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_u16_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&37_u16.to_le_bytes()).build();

        stream.write_value(37_u16, DEFAULT_PROTOCOL).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_read_u32_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&185851_u32.to_le_bytes()).build();

        assert_eq!(stream.read_value::<u32>(DEFAULT_PROTOCOL).await?, 185851);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_u32_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&185851_u32.to_le_bytes()).build();

        stream.write_value(185851_u32, DEFAULT_PROTOCOL).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_read_i32_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&(-185851_i32).to_le_bytes()).build();

        assert_eq!(
            stream.read_value::<i32>(DEFAULT_PROTOCOL).await?,
            -185851_i32
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_write_i32_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&(-185851_i32).to_le_bytes()).build();

        stream.write_value(-185851_i32, DEFAULT_PROTOCOL).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_read_usize_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&185851_u32.to_le_bytes()).build();

        assert_eq!(
            stream.read_value::<usize>(DEFAULT_PROTOCOL).await?,
            185851_usize
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_write_usize_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&185851_u32.to_le_bytes()).build();

        stream.write_value(185851_usize, DEFAULT_PROTOCOL).await?;

        Ok(())
    }
}
