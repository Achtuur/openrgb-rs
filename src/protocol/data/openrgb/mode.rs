use flagset::FlagSet;

use crate::data::ProtocolOption;
use crate::protocol::{DeserFromBuf, SerToBuf, WriteMessage};
use crate::OpenRgbError::ProtocolError;
use crate::{
    OpenRgbResult,
    protocol::data::{
        Color, ColorMode, Direction,
        ModeFlag::{self},
    },
};

/// RGB controller mode.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#mode-data) for more information.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModeData {
    /// Mode name.
    pub name: String,

    /// Mode value.
    pub value: i32,

    /// Mode flags set.
    pub flags: FlagSet<ModeFlag>,

    /// Mode minimum speed (if mode has [ModeFlag::HasSpeed] flag).
    pub speed_min: u32,

    /// Mode maximum speed (if mode has [ModeFlag::HasSpeed] flag).
    pub speed_max: u32,

    /// Mode maximum speed (if mode has [ModeFlag::HasSpeed] flag).
    pub speed: u32,

    /// Mode minimum brightness (if mode has [ModeFlag::HasBrightness] flag).
    ///
    /// Minimum protocol version: 3
    pub brightness_min: ProtocolOption<3, u32>,

    /// Mode maximum brightness (if mode has [ModeFlag::HasBrightness] flag).
    ///
    /// Minimum protocol version: 3
    pub brightness_max: ProtocolOption<3, u32>,

    /// Mode brightness (if mode has [ModeFlag::HasBrightness] flag).
    ///
    /// Minimum protocol version: 3
    pub brightness: ProtocolOption<3, u32>,

    /// Mode color mode.
    pub color_mode: ColorMode,

    /// Mode colors.
    pub colors: Vec<Color>,

    /// Mode minimum colors (if mode has non empty [Mode::colors] list).
    pub colors_min: u32,

    /// Mode minimum colors (if mode has non empty [Mode::colors] list).
    pub colors_max: u32,

    /// Mode direction.
    pub direction: Direction,

    /// Index of this mode, not part of received packet but set right after reading
    pub index: u32,
    // for use in self.size() as a workaround to not having the protocol version available there
    pub protocol_version: u32,
}

impl ModeData {
    pub fn brightness(&self) -> Option<u32> {
        match self.flags.contains(ModeFlag::HasBrightness) {
            true => self.brightness.value().copied(),
            false => None,
        }
    }

    pub fn set_brightness(&mut self, b: u32) {
        if self.flags.contains(ModeFlag::HasBrightness) {
            self.brightness.replace(b);
        }
    }

    pub fn brightness_min(&self) -> Option<u32> {
        match self.flags.contains(ModeFlag::HasBrightness) {
            true => self.brightness_min.value().copied(),
            false => None,
        }
    }

    pub fn brightness_max(&self) -> Option<u32> {
        match self.flags.contains(ModeFlag::HasBrightness) {
            true => self.brightness_max.value().copied(),
            false => None,
        }
    }

    pub fn speed(&self) -> Option<u32> {
        self.flags.contains(ModeFlag::HasSpeed).then_some(self.speed)
    }

    pub fn set_speed(&mut self, sp: u32) {
        if self.flags.contains(ModeFlag::HasSpeed) {
            self.speed = sp;
        }
    }

    pub fn speed_min(&self) -> Option<u32> {
        self.flags.contains(ModeFlag::HasSpeed).then_some(self.speed_min)
    }

    pub fn speed_max(&self) -> Option<u32> {
        self.flags.contains(ModeFlag::HasSpeed).then_some(self.speed_max)
    }

    pub fn direction(&self) -> Option<Direction> {
        self.flags.contains(ModeFlag::HasDirection).then_some(self.direction)
    }

    pub fn color_mode(&self) -> ColorMode {
        self.color_mode
    }

    pub fn colors(&self) -> &[Color] {
        &self.colors
    }

    pub fn colors_min(&self) -> Option<u32> {
        (!self.colors.is_empty()).then_some(self.colors_min)
    }

    pub fn colors_max(&self) -> Option<u32> {
        (!self.colors.is_empty()).then_some(self.colors_max)
    }
}

impl DeserFromBuf for ModeData {
    fn deserialize(buf: &mut crate::protocol::serialize::ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let name = buf.read_value()?;
        let value = buf.read_value()?;
        let flags = buf.read_value()?;
        let speed_min = buf.read_value()?;
        let speed_max = buf.read_value()?;
        let brightness_min = buf.read_value()?;
        let brightness_max = buf.read_value()?;
        let brightness = buf.read_value()?;
        let colors_min = buf.read_value()?;
        let colors_max = buf.read_value()?;
        let speed = buf.read_value()?;
        let direction = buf.read_value::<Direction>()?;
        let color_mode = buf.read_value()?;
        let colors = buf.read_value::<Vec<Color>>()?;

        Ok(ModeData {
            index: u32::MAX,
            protocol_version: buf.protocol_version(),
            name,
            value,
            flags,
            speed_min,
            speed_max,
            speed,
            brightness_min,
            brightness_max,
            brightness,
            colors_min,
            colors_max,
            direction,
            color_mode,
            colors,
        })
    }
}

impl SerToBuf for ModeData {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_value(&self.name)?;
        buf.write_value(&self.value)?;
        buf.write_value(&self.flags)?;
        buf.write_value(&self.speed_min)?;
        buf.write_value(&self.speed_max)?;
        buf.write_value(&self.brightness_min)?;
        buf.write_value(&self.brightness_max)?;
        buf.write_value(&self.brightness)?;
        buf.write_value(&self.colors_min)?;
        buf.write_value(&self.colors_max)?;
        buf.write_value(&self.speed)?;
        buf.write_value(&self.direction)?;
        buf.write_value(&self.color_mode)?;
        buf.write_value(&self.colors)?;
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::error::Error;

//     use tokio_test::io::Builder;

//     use crate::protocol::data::{Color, ColorMode, Direction, ModeData, ModeFlag::*};
//     use crate::protocol::tests::setup;
//     use crate::protocol::{ReadableStream, WritableStream};

//     #[tokio::test]
//     async fn test_read_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new()
//             .read(&5_u16.to_le_bytes()) // name len
//             .read(b"test\0") // name
//             .read(&46_i32.to_le_bytes()) // value
//             .read(&31_u32.to_le_bytes()) // flags
//             .read(&10_u32.to_le_bytes()) // speed_min
//             .read(&1000_u32.to_le_bytes()) // speed_max
//             .read(&1_u32.to_le_bytes()) // brightness_min
//             .read(&1024_u32.to_le_bytes()) // brightness_max
//             .read(&0_u32.to_le_bytes()) // colors_min
//             .read(&256_u32.to_le_bytes()) // colors_max
//             .read(&51_u32.to_le_bytes()) // speed
//             .read(&512_u32.to_le_bytes()) // brightness
//             .read(&4_u32.to_le_bytes()) // direction
//             .read(&1_u32.to_le_bytes()) // color_mode
//             .read(&2_u16.to_le_bytes()) // colors len
//             .read(&[37_u8, 54_u8, 126_u8, 0_u8]) // colors[0]
//             .read(&[37_u8, 54_u8, 255_u8, 0_u8]) // colors[1]
//             .build();

//         assert_eq!(
//             stream.read_value::<ModeData>().await?,
//             ModeData {
//                 protocol_version: 4,
//                 index: u32::MAX,
//                 name: "test".to_string(),
//                 value: 46,
//                 flags: HasDirection | HasSpeed | HasBrightness,
//                 speed_min: Some(10),
//                 speed_max: Some(1000),
//                 brightness_min: Some(1),
//                 brightness_max: Some(1024),
//                 colors_min: Some(0),
//                 colors_max: Some(256),
//                 speed: Some(51),
//                 brightness: Some(512),
//                 direction: Some(Direction::Horizontal),
//                 color_mode: Some(ColorMode::PerLED),
//                 colors: vec![
//                     Color {
//                         r: 37,
//                         g: 54,
//                         b: 126
//                     },
//                     Color {
//                         r: 37,
//                         g: 54,
//                         b: 255
//                     },
//                 ],
//             }
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_read_002() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new()
//             .read(&5_u16.to_le_bytes()) // name len
//             .read(b"test\0") // name
//             .read(&46_i32.to_le_bytes()) // value
//             .read(&0_u32.to_le_bytes()) // flags
//             .read(&10_u32.to_le_bytes()) // speed_min
//             .read(&1000_u32.to_le_bytes()) // speed_max
//             .read(&1_u32.to_le_bytes()) // brightness_min
//             .read(&1024_u32.to_le_bytes()) // brightness_max
//             .read(&0_u32.to_le_bytes()) // colors_min
//             .read(&256_u32.to_le_bytes()) // colors_max
//             .read(&51_u32.to_le_bytes()) // speed
//             .read(&512_u32.to_le_bytes()) // brightness
//             .read(&4_u32.to_le_bytes()) // direction
//             .read(&1_u32.to_le_bytes()) // color_mode
//             .read(&0_u16.to_le_bytes()) // colors len
//             .build();

//         assert_eq!(
//             stream.read_value::<ModeData>().await?,
//             ModeData {
//                 protocol_version: 4,
//                 index: u32::MAX,
//                 name: "test".to_string(),
//                 value: 46,
//                 flags: Default::default(),
//                 speed_min: None,
//                 speed_max: None,
//                 brightness_min: None,
//                 brightness_max: None,
//                 colors_min: None,
//                 colors_max: None,
//                 speed: None,
//                 brightness: None,
//                 direction: None,
//                 color_mode: Some(ColorMode::PerLED),
//                 colors: vec![],
//             }
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_read_003() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new()
//             .read(&5_u16.to_le_bytes()) // name len
//             .read(b"test\0") // name
//             .read(&46_i32.to_le_bytes()) // value
//             .read(&31_u32.to_le_bytes()) // flags
//             .read(&10_u32.to_le_bytes()) // speed_min
//             .read(&1000_u32.to_le_bytes()) // speed_max
//             .read(&0_u32.to_le_bytes()) // colors_min
//             .read(&256_u32.to_le_bytes()) // colors_max
//             .read(&51_u32.to_le_bytes()) // speed
//             .read(&4_u32.to_le_bytes()) // direction
//             .read(&1_u32.to_le_bytes()) // color_mode
//             .read(&2_u16.to_le_bytes()) // colors len
//             .read(&[37_u8, 54_u8, 126_u8, 0_u8]) // colors[0]
//             .read(&[37_u8, 54_u8, 255_u8, 0_u8]) // colors[1]
//             .build();

//         assert_eq!(
//             stream.read_value::<ModeData>().await?,
//             ModeData {
//                 protocol_version: 4,
//                 index: u32::MAX,
//                 name: "test".to_string(),
//                 value: 46,
//                 flags: HasDirection | HasSpeed | HasBrightness,
//                 speed_min: Some(10),
//                 speed_max: Some(1000),
//                 brightness_min: None,
//                 brightness_max: None,
//                 colors_min: Some(0),
//                 colors_max: Some(256),
//                 speed: Some(51),
//                 brightness: None,
//                 direction: Some(Direction::Horizontal),
//                 color_mode: Some(ColorMode::PerLED),
//                 colors: vec![
//                     Color {
//                         r: 37,
//                         g: 54,
//                         b: 126
//                     },
//                     Color {
//                         r: 37,
//                         g: 54,
//                         b: 255
//                     },
//                 ],
//             }
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new()
//             .write(&5_u16.to_le_bytes()) // name len
//             .write(b"test\0") // name
//             .write(&46_i32.to_le_bytes()) // value
//             .write(&31_u32.to_le_bytes()) // flags
//             .write(&10_u32.to_le_bytes()) // speed_min
//             .write(&1000_u32.to_le_bytes()) // speed_max
//             .write(&1_u32.to_le_bytes()) // brightness_min
//             .write(&1024_u32.to_le_bytes()) // brightness_max
//             .write(&0_u32.to_le_bytes()) // colors_min
//             .write(&256_u32.to_le_bytes()) // colors_max
//             .write(&51_u32.to_le_bytes()) // speed
//             .write(&512_u32.to_le_bytes()) // brightness
//             .write(&4_u32.to_le_bytes()) // direction
//             .write(&1_u32.to_le_bytes()) // color_mode
//             .write(&2_u16.to_le_bytes()) // colors len
//             .write(&[37_u8, 54_u8, 126_u8, 0_u8]) // colors[0]
//             .write(&[37_u8, 54_u8, 255_u8, 0_u8]) // colors[1]
//             .build();

//         stream
//             .write_value(&ModeData {
//                 protocol_version: 4,
//                 index: u32::MAX,
//                 name: "test".to_string(),
//                 value: 46,
//                 flags: HasDirection | HasSpeed | HasBrightness,
//                 speed_min: Some(10),
//                 speed_max: Some(1000),
//                 brightness_min: Some(1),
//                 brightness_max: Some(1024),
//                 colors_min: Some(0),
//                 colors_max: Some(256),
//                 speed: Some(51),
//                 brightness: Some(512),
//                 direction: Some(Direction::Horizontal),
//                 color_mode: Some(ColorMode::PerLED),
//                 colors: vec![
//                     Color {
//                         r: 37,
//                         g: 54,
//                         b: 126,
//                     },
//                     Color {
//                         r: 37,
//                         g: 54,
//                         b: 255,
//                     },
//                 ],
//             })
//             .await?;

//         Ok(())
//     }
// }
