use std::mem::size_of;

use crate::protocol::{ReadableStream, TryFromStream, Writable, WritableStream};
use crate::{OpenRgbError, OpenRgbResult};

impl Writable for () {
    fn size(&self) -> usize {
        0
    }

    async fn try_write(&self, _stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        Ok(0)
    }
}

impl TryFromStream for () {
    async fn try_read(_stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        Ok(())
    }
}

impl Writable for u8 {
    fn size(&self) -> usize {
        size_of::<u8>()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        stream.write_u8(*self).await?;
        Ok(size_of::<u8>())
    }
}

impl TryFromStream for u8 {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        stream.read_u8().await.map_err(Into::into)
    }
}

impl Writable for u16 {
    fn size(&self) -> usize {
        size_of::<u16>()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        stream.write_u16_le(*self).await?;
        Ok(size_of::<u16>())
    }
}

impl TryFromStream for u16 {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        stream.read_u16_le().await.map_err(Into::into)
    }
}

impl Writable for u32 {
    fn size(&self) -> usize {
        size_of::<u32>()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        stream.write_u32_le(*self).await?;
        Ok(size_of::<u32>())
    }
}

impl TryFromStream for u32 {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        stream.read_u32_le().await.map_err(Into::into)
    }
}

impl Writable for i32 {
    fn size(&self) -> usize {
        size_of::<i32>()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        stream.write_i32_le(*self).await?;
        Ok(size_of::<i32>())
    }
}

impl TryFromStream for i32 {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        stream.read_i32_le().await.map_err(Into::into)
    }
}

impl Writable for usize {
    fn size(&self) -> usize {
        size_of::<u32>()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        let val = u32::try_from(*self).map_err(|e| {
            OpenRgbError::ProtocolError(format!(
                "Data size is too large to encode: {} ({})",
                self, e
            ))
        })?;
        stream.write_value(&val).await
    }
}

impl TryFromStream for usize {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        stream.read_value::<u32>().await.map(|s| s as Self)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::protocol::tests::setup;
    use crate::protocol::{ReadableStream, WritableStream};

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

        stream.write_value(&()).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_read_u8_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&[37_u8]).build();

        assert_eq!(stream.read_value::<u8>().await?, 37);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_u8_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&[37_u8]).build();

        stream.write_value(&37_u8).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_read_u16_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&37_u16.to_le_bytes()).build();

        assert_eq!(stream.read_value::<u16>().await?, 37);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_u16_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&37_u16.to_le_bytes()).build();

        stream.write_value(&37_u16).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_read_u32_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&185851_u32.to_le_bytes()).build();

        assert_eq!(stream.read_value::<u32>().await?, 185851);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_u32_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&185851_u32.to_le_bytes()).build();

        stream.write_value(&185851_u32).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_read_i32_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&(-185851_i32).to_le_bytes()).build();

        assert_eq!(stream.read_value::<i32>().await?, -185851_i32);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_i32_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&(-185851_i32).to_le_bytes()).build();

        stream.write_value(&-185851_i32).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_read_usize_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&185851_u32.to_le_bytes()).build();

        assert_eq!(stream.read_value::<usize>().await?, 185851_usize);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_usize_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&185851_u32.to_le_bytes()).build();

        stream.write_value(&185851_usize).await?;

        Ok(())
    }
}
