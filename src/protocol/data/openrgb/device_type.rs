use crate::protocol::{DeserFromBuf, SerToBuf};
use crate::{impl_enum_discriminant, OpenRgbResult, ReceivedMessage, WriteMessage};

/// RGB controller device type.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
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
    Unknown = -1,
}

impl_enum_discriminant!(DeviceType,
    Motherboard: 0,
    DRam: 1,
    Gpu: 2,
    Cooler: 3,
    LEDStrip: 4,
    Keyboard: 5,
    Mouse: 6,
    MouseMat: 7,
    Headset: 8,
    HeadsetStand: 9,
    Gamepad: 10,
    Light: 11,
    Speaker: 12,
    Virtual: 13,
    Unknown: 14
);

impl DeserFromBuf for DeviceType {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let device_type_raw = buf.read_u32()?;
        let device_type = DeviceType::try_from(device_type_raw).unwrap_or(DeviceType::Unknown);
        Ok(device_type)
    }
}

impl SerToBuf for DeviceType {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        let num = u32::from(self);
        buf.write_u32(num);
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::error::Error;

//     use tokio_test::io::Builder;

//     use crate::protocol::data::DeviceType;
//     use crate::protocol::tests::setup;

//     #[tokio::test]
//     async fn test_read_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().read(&8_u32.to_le_bytes()).build();

//         assert_eq!(
//             stream.read_value::<DeviceType>().await?,
//             DeviceType::Headset
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().write(&8_u32.to_le_bytes()).build();

//         stream.write_value(&DeviceType::Headset).await?;

//         Ok(())
//     }
// }
