use crate::protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};
use crate::{impl_enum_discriminant, OpenRgbError, OpenRgbResult};

/// RGB controller color mode.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
#[derive(Eq, PartialEq, Debug, Copy, Clone, Default)]
pub enum ColorMode {
    /// No color mode.
    #[default]
    None = 0,

    /// Per LED colors.
    PerLED = 1,

    /// Mode specific colors.
    ModeSpecific = 2,

    /// Random colors.
    Random = 3,
}

impl_enum_discriminant!(ColorMode, None: 0, PerLED: 1, ModeSpecific: 2, Random: 3);

impl SerToBuf for ColorMode {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        let num = u32::from(self);
        buf.write_u32(num);
        Ok(())
    }
}

impl DeserFromBuf for ColorMode {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let raw = buf.read_u32()?;
        ColorMode::try_from(raw)
    }
}

// #[cfg(test)]
// mod tests {
//     use std::error::Error;

//     use tokio_test::io::Builder;

//     use crate::protocol::{data::ColorMode, tests::setup};

//     #[tokio::test]
//     async fn test_read_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().read(&3u32.to_le_bytes()).build();

//         assert_eq!(stream.read_value::<ColorMode>().await?, ColorMode::Random);

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().write(&3u32.to_le_bytes()).build();

//         stream.write_value(&ColorMode::Random).await?;

//         Ok(())
//     }
// }
