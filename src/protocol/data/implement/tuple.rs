use crate::OpenRgbResult;
use crate::protocol::{ReadableStream, TryFromStream, Writable, WritableStream};

impl<A: Writable, B: Writable> Writable for (A, B) {
    fn size(&self) -> usize {
        self.0.size() + self.1.size()
    }

    async fn try_write(self, stream: &mut impl WritableStream) -> OpenRgbResult<()> {
        stream.write_value(self.0).await?;
        stream.write_value(self.1).await?;
        Ok(())
    }
}

impl<A: TryFromStream, B: TryFromStream> TryFromStream for (A, B) {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        Ok((
            stream.read_value::<A>().await?,
            stream.read_value::<B>().await?,
        ))
    }
}

impl<A: Writable, B: Writable, C: Writable> Writable for (A, B, C) {
    fn size(&self) -> usize {
        self.0.size() + self.1.size() + self.2.size()
    }

    async fn try_write(self, stream: &mut impl WritableStream) -> OpenRgbResult<()> {
        stream.write_value(self.0).await?;
        stream.write_value(self.1).await?;
        stream.write_value(self.2).await?;
        Ok(())
    }
}

impl<A: TryFromStream, B: TryFromStream, C: TryFromStream> TryFromStream for (A, B, C) {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        Ok((
            stream.read_value::<A>().await?,
            stream.read_value::<B>().await?,
            stream.read_value::<C>().await?,
        ))
    }
}

impl<A: Writable, B: Writable, C: Writable, D: Writable> Writable for (A, B, C, D) {
    fn size(&self) -> usize {
        self.0.size() + self.1.size() + self.2.size() + self.3.size()
    }

    async fn try_write(self, stream: &mut impl WritableStream) -> OpenRgbResult<()> {
        stream.write_value(self.0).await?;
        stream.write_value(self.1).await?;
        stream.write_value(self.2).await?;
        stream.write_value(self.3).await?;
        Ok(())
    }
}

impl<A: TryFromStream, B: TryFromStream, C: TryFromStream, D: TryFromStream> TryFromStream
    for (A, B, C, D)
{
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        Ok((
            stream.read_value::<A>().await?,
            stream.read_value::<B>().await?,
            stream.read_value::<C>().await?,
            stream.read_value::<D>().await?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::protocol::data::DeviceType;
    use crate::protocol::tests::setup;
    use crate::protocol::{ReadableStream, WritableStream};

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&37_u8.to_le_bytes())
            .read(&1337_u32.to_le_bytes())
            .read(&(-1337_i32).to_le_bytes())
            .read(&4_u32.to_le_bytes())
            .build();

        assert_eq!(
            stream.read_value::<(u8, u32, i32, DeviceType)>().await?,
            (37, 1337, -1337, DeviceType::LEDStrip)
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(&37_u8.to_le_bytes())
            .write(&1337_u32.to_le_bytes())
            .write(&(-1337_i32).to_le_bytes())
            .write(&4_u32.to_le_bytes())
            .build();

        stream
            .write_value((37_u8, 1337_u32, (-1337_i32), DeviceType::LEDStrip))
            .await?;

        Ok(())
    }
}
