use std::mem::size_of;

use rgb::RGB8;

use crate::OpenRgbResult;
use crate::protocol::{ReadableStream, TryFromStream, Writable, WritableStream};

/// RGB controller color, aliased to [rgb] crate's [RGB8] type.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
pub type Color = RGB8;

impl TryFromStream for Color {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        let r = stream.read_value().await?;
        let g = stream.read_value().await?;
        let b = stream.read_value().await?;
        let _padding = stream.read_value::<u8>().await?;
        Ok(Color { r, g, b })
    }
}

impl Writable for Color {
    fn size(&self) -> usize {
        4 * size_of::<u8>()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<()> {
        stream.write_value(&self.r).await?;
        stream.write_value(&self.g).await?;
        stream.write_value(&self.b).await?;
        stream.write_value(&0u8).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::protocol::data::Color;
    use crate::protocol::tests::setup;
    use crate::protocol::{ReadableStream, WritableStream};

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&[37_u8, 54_u8, 126_u8, 0_u8]).build();

        assert_eq!(
            stream.read_value::<Color>().await?,
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
            .write_value(&Color {
                r: 37,
                g: 54,
                b: 126,
            })
            .await?;

        Ok(())
    }
}
