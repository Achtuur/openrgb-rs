use std::mem::size_of;

use num_traits::FromPrimitive;

use crate::OpenRgbResult;
use crate::protocol::{ReadableStream, WritableStream};
use crate::protocol::{TryFromStream, Writable};

/// RGB controller device type.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
#[derive(Primitive, Eq, PartialEq, Debug, Copy, Clone)]
pub enum DeviceType {
    /// Motherboard.
    Motherboard = 0,

    /// DRAM.
    DRam = 1,

    /// GPU.
    Gpu = 2,

    /// Cooler.
    Cooler = 3,

    /// LED strip.
    LEDStrip = 4,

    /// Keyboard.
    Keyboard = 5,

    /// Mouse.
    Mouse = 6,

    /// Mouse mat.
    MouseMat = 7,

    /// Headset.
    Headset = 8,

    /// Headset stand.
    HeadsetStand = 9,

    /// Gamepad.
    Gamepad = 10,

    /// Light.
    Light = 11,

    /// Speaker.
    Speaker = 12,

    /// Virtual.
    Virtual = 13,

    /// Unknown.
    Unknown = 14,
}

impl Writable for DeviceType {
    fn size(&self) -> usize {
        size_of::<u32>()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        let num = *self as u32;
        stream.write_value(&num).await
    }
}

impl TryFromStream for DeviceType {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        let device_type_raw = stream.read_value().await?;
        let device_type = DeviceType::from_u32(device_type_raw).unwrap_or(DeviceType::Unknown);
        Ok(device_type)
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

        let mut stream = Builder::new().read(&8_u32.to_le_bytes()).build();

        assert_eq!(
            stream.read_value::<DeviceType>().await?,
            DeviceType::Headset
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&8_u32.to_le_bytes()).build();

        stream.write_value(&DeviceType::Headset).await?;

        Ok(())
    }
}
