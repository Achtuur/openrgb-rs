use crate::data::{TryFromStream, Writable};
use crate::protocol::{ReadableStream, WritableStream};
use crate::OpenRgbError;

impl<A: Writable, B: Writable> Writable for (A, B) {
    fn size(&self, protocol: u32) -> usize {
        self.0.size(protocol) + self.1.size(protocol)
    }

    async fn try_write(
        self,
        stream: &mut impl WritableStream,
        protocol: u32,
    ) -> Result<(), OpenRgbError> {
        stream.write_value(self.0, protocol).await?;
        stream.write_value(self.1, protocol).await?;
        Ok(())
    }
}

impl<A: TryFromStream, B: TryFromStream> TryFromStream for (A, B) {
    async fn try_read(
        stream: &mut impl ReadableStream,
        protocol: u32,
    ) -> Result<Self, OpenRgbError> {
        Ok((
            stream.read_value::<A>(protocol).await?,
            stream.read_value::<B>(protocol).await?,
        ))
    }
}

impl<A: Writable, B: Writable, C: Writable> Writable for (A, B, C) {
    fn size(&self, protocol: u32) -> usize {
        self.0.size(protocol) + self.1.size(protocol) + self.2.size(protocol)
    }

    async fn try_write(
        self,
        stream: &mut impl WritableStream,
        protocol: u32,
    ) -> Result<(), OpenRgbError> {
        stream.write_value(self.0, protocol).await?;
        stream.write_value(self.1, protocol).await?;
        stream.write_value(self.2, protocol).await?;
        Ok(())
    }
}

impl<A: TryFromStream, B: TryFromStream, C: TryFromStream> TryFromStream for (A, B, C) {
    async fn try_read(
        stream: &mut impl ReadableStream,
        protocol: u32,
    ) -> Result<Self, OpenRgbError> {
        Ok((
            stream.read_value::<A>(protocol).await?,
            stream.read_value::<B>(protocol).await?,
            stream.read_value::<C>(protocol).await?,
        ))
    }
}

impl<A: Writable, B: Writable, C: Writable, D: Writable> Writable
    for (A, B, C, D)
{
    fn size(&self, protocol: u32) -> usize {
        self.0.size(protocol)
            + self.1.size(protocol)
            + self.2.size(protocol)
            + self.3.size(protocol)
    }

    async fn try_write(
        self,
        stream: &mut impl WritableStream,
        protocol: u32,
    ) -> Result<(), OpenRgbError> {
        stream.write_value(self.0, protocol).await?;
        stream.write_value(self.1, protocol).await?;
        stream.write_value(self.2, protocol).await?;
        stream.write_value(self.3, protocol).await?;
        Ok(())
    }
}

impl<A: TryFromStream, B: TryFromStream, C: TryFromStream, D: TryFromStream> TryFromStream
    for (A, B, C, D)
{
    async fn try_read(
        stream: &mut impl ReadableStream,
        protocol: u32,
    ) -> Result<Self, OpenRgbError> {
        Ok((
            stream.read_value::<A>(protocol).await?,
            stream.read_value::<B>(protocol).await?,
            stream.read_value::<C>(protocol).await?,
            stream.read_value::<D>(protocol).await?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::data::DeviceType;
    use crate::protocol::{ReadableStream, WritableStream};
    use crate::tests::setup;
    use crate::DEFAULT_PROTOCOL;

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
            stream
                .read_value::<(u8, u32, i32, DeviceType)>(DEFAULT_PROTOCOL)
                .await?,
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
            .write_value(
                (37_u8, 1337_u32, (-1337_i32), DeviceType::LEDStrip),
                DEFAULT_PROTOCOL,
            )
            .await?;

        Ok(())
    }
}
