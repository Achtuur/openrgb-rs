use std::mem::size_of;

use rgb::RGB8;

use crate::data::{TryFromStream, Writable};
use crate::protocol::{ReadableStream, WritableStream};
use crate::{OpenRgbError, OpenRgbResult};

/// RGB controller color, aliased to [rgb] crate's [RGB8] type.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
pub type Color = RGB8;

impl TryFromStream for Color {
    async fn try_read(
        stream: &mut impl ReadableStream,
        protocol: u32,
    ) -> Result<Self, OpenRgbError> {
        let r = stream.read_value(protocol).await?;
        let g = stream.read_value(protocol).await?;
        let b = stream.read_value(protocol).await?;
        let _padding = stream.read_value::<u8>(protocol).await?;
        Ok(Color { r, g, b })
    }
}

impl Writable for Color {
    fn size(&self, _protocol: u32) -> usize {
        4 * size_of::<u8>()
    }

    async fn try_write(
        self,
        stream: &mut impl WritableStream,
        protocol: u32,
    ) -> OpenRgbResult<()> {
        stream.write_value(self.r, protocol).await?;
        stream.write_value(self.g, protocol).await?;
        stream.write_value(self.b, protocol).await?;
        stream.write_value(0u8, protocol).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::data::Color;
    use crate::protocol::{ReadableStream, WritableStream};
    use crate::tests::setup;
    use crate::DEFAULT_PROTOCOL;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&[37_u8, 54_u8, 126_u8, 0_u8]).build();

        assert_eq!(
            stream.read_value::<Color>(DEFAULT_PROTOCOL).await?,
            Color {
                r: 37,
                g: 54,
                b: 126
            }
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&[37_u8, 54_u8, 126_u8, 0_u8]).build();

        stream
            .write_value(
                Color {
                    r: 37,
                    g: 54,
                    b: 126,
                },
                DEFAULT_PROTOCOL,
            )
            .await?;

        Ok(())
    }
}
