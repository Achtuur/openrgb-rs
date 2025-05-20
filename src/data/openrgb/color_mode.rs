use std::mem::size_of;

use num_traits::FromPrimitive;

use crate::data::{TryFromStream, Writable};
use crate::protocol::{ReadableStream, WritableStream};
use crate::OpenRgbError;
use crate::OpenRgbError::ProtocolError;

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
    fn size(&self, _protocol: u32) -> usize {
        size_of::<u32>()
    }

    async fn try_write(
        self,
        stream: &mut impl WritableStream,
        protocol: u32,
    ) -> Result<(), OpenRgbError> {
        stream.write_value(self as u32, protocol).await
    }
}

impl TryFromStream for ColorMode {
    async fn try_read(
        stream: &mut impl ReadableStream,
        protocol: u32,
    ) -> Result<Self, OpenRgbError> {
        stream.read_value(protocol).await.and_then(|id| {
            ColorMode::from_u32(id)
                .ok_or_else(|| ProtocolError(format!("unknown color mode \"{}\"", id)))
        })
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
