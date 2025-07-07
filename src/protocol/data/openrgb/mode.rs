use flagset::FlagSet;
use num_traits::FromPrimitive;

use crate::OpenRgbError::ProtocolError;
use crate::protocol::{ReadableStream, WritableStream};
use crate::{
    OpenRgbResult,
    protocol::data::{
        Color, ColorMode, Direction,
        ModeFlag::{self, *},
    },
    protocol::{TryFromStream, Writable},
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
    pub speed_min: Option<u32>,

    /// Mode maximum speed (if mode has [ModeFlag::HasSpeed] flag).
    pub speed_max: Option<u32>,

    /// Mode maximum speed (if mode has [ModeFlag::HasSpeed] flag).
    pub speed: Option<u32>,

    /// Mode minimum brightness (if mode has [ModeFlag::HasBrightness] flag).
    ///
    /// Minimum protocol version: 3
    pub brightness_min: Option<u32>,

    /// Mode maximum brightness (if mode has [ModeFlag::HasBrightness] flag).
    ///
    /// Minimum protocol version: 3
    pub brightness_max: Option<u32>,

    /// Mode brightness (if mode has [ModeFlag::HasBrightness] flag).
    ///
    /// Minimum protocol version: 3
    pub brightness: Option<u32>,

    /// Mode color mode.
    pub color_mode: Option<ColorMode>,

    /// Mode colors.
    pub colors: Vec<Color>,

    /// Mode minimum colors (if mode has non empty [Mode::colors] list).
    pub colors_min: Option<u32>,

    /// Mode minimum colors (if mode has non empty [Mode::colors] list).
    pub colors_max: Option<u32>,

    /// Mode direction.
    pub direction: Option<Direction>,

    /// Index of this mode, not part of received packet but set right after reading
    pub index: u32,
    // for use in self.size() as a workaround to not having the protocol version available there
    pub protocol_version: u32,
}

impl TryFromStream for ModeData {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        let name = stream.read_value().await?;
        let value = stream.read_value().await?;
        let flags = stream.read_value().await?;
        let speed_min = stream.read_value().await?;
        let speed_max = stream.read_value().await?;
        let brightness_min = stream.read_value_min_version(3).await?;
        let brightness_max = stream.read_value_min_version(3).await?;
        let brightness = stream.read_value_min_version(3).await?;
        let colors_min = stream.read_value().await?;
        let colors_max = stream.read_value().await?;
        let speed = stream.read_value().await?;
        let direction = stream.read_value().await?;
        let color_mode = stream.read_value().await?;
        let colors = stream.read_value::<Vec<Color>>().await?;

        Ok(ModeData {
            index: u32::MAX,
            protocol_version: stream.protocol_version(),
            name,
            value,
            flags,
            speed_min: flags.contains(HasSpeed).then_some(speed_min),
            speed_max: flags.contains(HasSpeed).then_some(speed_max),
            speed: flags.contains(HasSpeed).then_some(speed),
            brightness_min: if flags.contains(HasBrightness) {
                brightness_min
            } else {
                None
            },
            brightness_max: if flags.contains(HasBrightness) {
                brightness_max
            } else {
                None
            },
            brightness: if flags.contains(HasBrightness) {
                brightness
            } else {
                None
            },
            colors_min: if colors.is_empty() {
                None
            } else {
                Some(colors_min)
            },
            colors_max: if colors.is_empty() {
                None
            } else {
                Some(colors_max)
            },
            direction: if flags.contains(HasDirection) {
                Some(
                    Direction::from_u32(direction).ok_or_else(|| {
                        ProtocolError(format!("unknown direction \"{}\"", direction))
                    })?,
                )
            } else {
                None
            },
            color_mode: Some(color_mode),
            colors,
        })
    }
}

impl Writable for ModeData {
    fn size(&self) -> usize {
        let mut size = 0;
        size += self.name.size();
        size += self.value.size();
        size += self.flags.size();
        size += self.speed_min.unwrap_or_default().size();
        size += self.speed_max.unwrap_or_default().size();
        if self.protocol_version >= 3 {
            size += self.brightness_min.unwrap_or_default().size();
            size += self.brightness_max.unwrap_or_default().size();
            size += self.brightness.unwrap_or_default().size();
        }
        size += self.colors_min.unwrap_or_default().size();
        size += self.colors_max.unwrap_or_default().size();
        size += self.speed.unwrap_or_default().size();
        size += self.direction.unwrap_or_default().size();
        size += self.color_mode.unwrap_or_default().size();
        size += self.colors.size();
        size
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        let mut n = 0;
        n += stream.write_value(&self.name).await?;
        n += stream.write_value(&self.value).await?;
        n += stream.write_value(&self.flags).await?;
        n += stream
            .write_value(&self.speed_min.unwrap_or_default())
            .await?;
        n += stream
            .write_value(&self.speed_max.unwrap_or_default())
            .await?;
        n += stream
            .write_value_min_version(&self.brightness_min.unwrap_or_default(), 3)
            .await?;
        n += stream
            .write_value_min_version(&self.brightness_max.unwrap_or_default(), 3)
            .await?;
        n += stream
            .write_value_min_version(&self.brightness.unwrap_or_default(), 3)
            .await?;
        n += stream
            .write_value(&self.colors_min.unwrap_or_default())
            .await?;
        n += stream
            .write_value(&self.colors_max.unwrap_or_default())
            .await?;
        n += stream.write_value(&self.speed.unwrap_or_default()).await?;
        n += stream
            .write_value(&self.direction.unwrap_or_default())
            .await?;
        n += stream
            .write_value(&self.color_mode.unwrap_or_default())
            .await?;
        n += stream.write_value(&self.colors).await?;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::protocol::data::{Color, ColorMode, Direction, ModeData, ModeFlag::*};
    use crate::protocol::tests::setup;
    use crate::protocol::{ReadableStream, WritableStream};

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&5_u16.to_le_bytes()) // name len
            .read(b"test\0") // name
            .read(&46_i32.to_le_bytes()) // value
            .read(&31_u32.to_le_bytes()) // flags
            .read(&10_u32.to_le_bytes()) // speed_min
            .read(&1000_u32.to_le_bytes()) // speed_max
            .read(&1_u32.to_le_bytes()) // brightness_min
            .read(&1024_u32.to_le_bytes()) // brightness_max
            .read(&0_u32.to_le_bytes()) // colors_min
            .read(&256_u32.to_le_bytes()) // colors_max
            .read(&51_u32.to_le_bytes()) // speed
            .read(&512_u32.to_le_bytes()) // brightness
            .read(&4_u32.to_le_bytes()) // direction
            .read(&1_u32.to_le_bytes()) // color_mode
            .read(&2_u16.to_le_bytes()) // colors len
            .read(&[37_u8, 54_u8, 126_u8, 0_u8]) // colors[0]
            .read(&[37_u8, 54_u8, 255_u8, 0_u8]) // colors[1]
            .build();

        assert_eq!(
            stream.read_value::<ModeData>().await?,
            ModeData {
                protocol_version: 4,
                index: u32::MAX,
                name: "test".to_string(),
                value: 46,
                flags: HasDirection | HasSpeed | HasBrightness,
                speed_min: Some(10),
                speed_max: Some(1000),
                brightness_min: Some(1),
                brightness_max: Some(1024),
                colors_min: Some(0),
                colors_max: Some(256),
                speed: Some(51),
                brightness: Some(512),
                direction: Some(Direction::Horizontal),
                color_mode: Some(ColorMode::PerLED),
                colors: vec![
                    Color {
                        r: 37,
                        g: 54,
                        b: 126
                    },
                    Color {
                        r: 37,
                        g: 54,
                        b: 255
                    },
                ],
            }
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_read_002() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&5_u16.to_le_bytes()) // name len
            .read(b"test\0") // name
            .read(&46_i32.to_le_bytes()) // value
            .read(&0_u32.to_le_bytes()) // flags
            .read(&10_u32.to_le_bytes()) // speed_min
            .read(&1000_u32.to_le_bytes()) // speed_max
            .read(&1_u32.to_le_bytes()) // brightness_min
            .read(&1024_u32.to_le_bytes()) // brightness_max
            .read(&0_u32.to_le_bytes()) // colors_min
            .read(&256_u32.to_le_bytes()) // colors_max
            .read(&51_u32.to_le_bytes()) // speed
            .read(&512_u32.to_le_bytes()) // brightness
            .read(&4_u32.to_le_bytes()) // direction
            .read(&1_u32.to_le_bytes()) // color_mode
            .read(&0_u16.to_le_bytes()) // colors len
            .build();

        assert_eq!(
            stream.read_value::<ModeData>().await?,
            ModeData {
                protocol_version: 4,
                index: u32::MAX,
                name: "test".to_string(),
                value: 46,
                flags: Default::default(),
                speed_min: None,
                speed_max: None,
                brightness_min: None,
                brightness_max: None,
                colors_min: None,
                colors_max: None,
                speed: None,
                brightness: None,
                direction: None,
                color_mode: Some(ColorMode::PerLED),
                colors: vec![],
            }
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_read_003() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&5_u16.to_le_bytes()) // name len
            .read(b"test\0") // name
            .read(&46_i32.to_le_bytes()) // value
            .read(&31_u32.to_le_bytes()) // flags
            .read(&10_u32.to_le_bytes()) // speed_min
            .read(&1000_u32.to_le_bytes()) // speed_max
            .read(&0_u32.to_le_bytes()) // colors_min
            .read(&256_u32.to_le_bytes()) // colors_max
            .read(&51_u32.to_le_bytes()) // speed
            .read(&4_u32.to_le_bytes()) // direction
            .read(&1_u32.to_le_bytes()) // color_mode
            .read(&2_u16.to_le_bytes()) // colors len
            .read(&[37_u8, 54_u8, 126_u8, 0_u8]) // colors[0]
            .read(&[37_u8, 54_u8, 255_u8, 0_u8]) // colors[1]
            .build();

        assert_eq!(
            stream.read_value::<ModeData>().await?,
            ModeData {
                protocol_version: 4,
                index: u32::MAX,
                name: "test".to_string(),
                value: 46,
                flags: HasDirection | HasSpeed | HasBrightness,
                speed_min: Some(10),
                speed_max: Some(1000),
                brightness_min: None,
                brightness_max: None,
                colors_min: Some(0),
                colors_max: Some(256),
                speed: Some(51),
                brightness: None,
                direction: Some(Direction::Horizontal),
                color_mode: Some(ColorMode::PerLED),
                colors: vec![
                    Color {
                        r: 37,
                        g: 54,
                        b: 126
                    },
                    Color {
                        r: 37,
                        g: 54,
                        b: 255
                    },
                ],
            }
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(&5_u16.to_le_bytes()) // name len
            .write(b"test\0") // name
            .write(&46_i32.to_le_bytes()) // value
            .write(&31_u32.to_le_bytes()) // flags
            .write(&10_u32.to_le_bytes()) // speed_min
            .write(&1000_u32.to_le_bytes()) // speed_max
            .write(&1_u32.to_le_bytes()) // brightness_min
            .write(&1024_u32.to_le_bytes()) // brightness_max
            .write(&0_u32.to_le_bytes()) // colors_min
            .write(&256_u32.to_le_bytes()) // colors_max
            .write(&51_u32.to_le_bytes()) // speed
            .write(&512_u32.to_le_bytes()) // brightness
            .write(&4_u32.to_le_bytes()) // direction
            .write(&1_u32.to_le_bytes()) // color_mode
            .write(&2_u16.to_le_bytes()) // colors len
            .write(&[37_u8, 54_u8, 126_u8, 0_u8]) // colors[0]
            .write(&[37_u8, 54_u8, 255_u8, 0_u8]) // colors[1]
            .build();

        stream
            .write_value(&ModeData {
                protocol_version: 4,
                index: u32::MAX,
                name: "test".to_string(),
                value: 46,
                flags: HasDirection | HasSpeed | HasBrightness,
                speed_min: Some(10),
                speed_max: Some(1000),
                brightness_min: Some(1),
                brightness_max: Some(1024),
                colors_min: Some(0),
                colors_max: Some(256),
                speed: Some(51),
                brightness: Some(512),
                direction: Some(Direction::Horizontal),
                color_mode: Some(ColorMode::PerLED),
                colors: vec![
                    Color {
                        r: 37,
                        g: 54,
                        b: 126,
                    },
                    Color {
                        r: 37,
                        g: 54,
                        b: 255,
                    },
                ],
            })
            .await?;

        Ok(())
    }
}
