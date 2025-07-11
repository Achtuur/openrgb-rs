use rgb::RGB8;

use crate::OpenRgbResult;
use crate::protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};

/// RGB controller color, aliased to [rgb] crate's [RGB8] type.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
pub type Color = RGB8;

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

// #[cfg(test)]
// mod tests {
//     use std::error::Error;

//     use tokio_test::io::Builder;

//     use crate::protocol::data::Color;
//     use crate::protocol::tests::setup;

//     #[tokio::test]
//     async fn test_read_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().read(&[37_u8, 54_u8, 126_u8, 0_u8]).build();

//         assert_eq!(
//             stream.read_value::<Color>().await?,
//             Color {
//                 r: 37,
//                 g: 54,
//                 b: 126
//             }
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().write(&[37_u8, 54_u8, 126_u8, 0_u8]).build();

//         stream
//             .write_value(&Color {
//                 r: 37,
//                 g: 54,
//                 b: 126,
//             })
//             .await?;

//         Ok(())
//     }
// }
