use std::mem::size_of;

use num_traits::FromPrimitive;

use crate::protocol::{ReadableStream, TryFromStream, Writable, WritableStream};
use crate::{OpenRgbError, OpenRgbResult};


/// RGB controller color mode.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
#[derive(Primitive, Eq, PartialEq, Debug, Copy, Clone)]
pub enum ColorMode {
    /// No color mode.
    None = 0,

    /// Per LED colors.
    PerLED = 1,

    /// Mode specific colors.
    ModeSpecific = 2,

    /// Random colors.
    Random = 3,
}

impl Default for ColorMode {
    fn default() -> Self {
        ColorMode::None
    }
}

impl Writable for ColorMode {
    fn size(&self) -> usize {
        size_of::<u32>()
    }

    async fn try_write(
        self,
        stream: &mut impl WritableStream,
    ) -> OpenRgbResult<()> {
        stream.write_value(self as u32).await
    }
}

impl TryFromStream for ColorMode {
    async fn try_read(
        stream: &mut impl ReadableStream,
    ) -> OpenRgbResult<Self> {
        let raw = stream.read_value().await?;
        ColorMode::from_u32(raw)
        .ok_or(OpenRgbError::ProtocolError(format!("unknown color mode \"{}\"", raw)))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::data::ColorMode;
    use crate::protocol::{ReadableStream, WritableStream};
    use crate::tests::setup;
    use crate::DEFAULT_PROTOCOL;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&3u32.to_le_bytes()).build();

        assert_eq!(
            stream.read_value::<ColorMode>(DEFAULT_PROTOCOL).await?,
            ColorMode::Random
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&3u32.to_le_bytes()).build();

        stream
            .write_value(ColorMode::Random, DEFAULT_PROTOCOL)
            .await?;

        Ok(())
    }
}
