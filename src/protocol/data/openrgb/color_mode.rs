use std::mem::size_of;

use num_traits::FromPrimitive;

use crate::protocol::stream2::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};
use crate::protocol::{ReadableStream, TryFromStream, Writable, WritableStream};
use crate::{OpenRgbError, OpenRgbResult};

/// RGB controller color mode.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
#[derive(Primitive, Eq, PartialEq, Debug, Copy, Clone, Default)]
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

impl Writable for ColorMode {
    fn size(&self) -> usize {
        size_of::<u32>()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        let num = *self as u32;
        stream.write_value(&num).await
    }
}

impl SerToBuf for ColorMode {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        let num = *self as u32;
        buf.write_u32(num);
        Ok(())
    }
}

impl TryFromStream for ColorMode {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        let raw = stream.read_value().await?;
        ColorMode::from_u32(raw).ok_or(OpenRgbError::ProtocolError(format!(
            "unknown color mode \"{}\"",
            raw
        )))
    }
}

impl DeserFromBuf for ColorMode {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let raw = buf.read_u32()?;
        ColorMode::from_u32(raw).ok_or(OpenRgbError::ProtocolError(format!(
            "unknown color mode \"{}\"",
            raw
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::protocol::{ReadableStream, WritableStream, data::ColorMode, tests::setup};

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&3u32.to_le_bytes()).build();

        assert_eq!(stream.read_value::<ColorMode>().await?, ColorMode::Random);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&3u32.to_le_bytes()).build();

        stream.write_value(&ColorMode::Random).await?;

        Ok(())
    }
}
