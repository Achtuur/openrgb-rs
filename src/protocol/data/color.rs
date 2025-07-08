use std::mem::size_of;

use rgb::RGB8;

use crate::protocol::stream2::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};
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
        let _ = stream.read_value::<u8>().await?;
        Ok(Color { r, g, b })
    }
}

impl Writable for Color {
    fn size(&self) -> usize {
        4 * size_of::<u8>()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        let s= stream.write_value(&self.r).await?;
        let s1= stream.write_value(&self.g).await?;
        let s2= stream.write_value(&self.b).await?;
        let s3= stream.write_value(&0u8).await?;
        Ok(s + s1 + s2 + s3)
    }
}

impl DeserFromBuf for Color {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let r = buf.read_u8()?;
        let g = buf.read_u8()?;
        let b = buf.read_u8()?;
        let _ = buf.read_u8()?; // Skip the alpha channel
        Ok(Color { r, g, b })
    }
}

impl SerToBuf for Color {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u8(self.r);
        buf.write_u8(self.g);
        buf.write_u8(self.b);
        buf.write_u8(0u8); // Skip the alpha channel
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
