use array2d::Array2D;

use crate::OpenRgbResult;
use crate::protocol::data::ZoneType;
use crate::protocol::{ReadableStream, TryFromStream};

use super::SegmentData;

/// RGB controller zone.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#zone-data) for more information.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ZoneData {
    /// Id of this zone.
    ///
    /// Not part of the packet, but set right after reading
    /// since the sender knows the zone id.
    pub id: u32,
    /// Zone name.
    pub name: String,

    /// Zone type.
    pub zone_type: ZoneType,

    /// Zone minimum LED number.
    pub leds_min: u32,

    /// Zone maximum LED number.
    pub leds_max: u32,

    /// Zone LED count.
    pub leds_count: u32,

    /// Segments in this zone
    ///
    /// Minimum version: 4
    pub segments: Vec<SegmentData>,

    /// Flags for this zone.
    ///
    /// Minimum version: 5
    pub flags: Option<u32>,

    /// Zone LED matrix (if [Zone::type] is [ZoneType::Matrix]).
    ///
    /// Matrix is the "position" of the LEDs in the zone relative to the top left corner.
    ///
    /// The value represents the LED id of the LED at that position.
    /// A value of `u32::MAX` means that there is no led present.
    pub matrix: Option<Array2D<u32>>,
}

impl TryFromStream for ZoneData {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        let name = stream.read_value().await?;
        let zone_type = stream.read_value().await?;
        let leds_min = stream.read_value().await?;
        let leds_max = stream.read_value().await?;
        let leds_count = stream.read_value().await?;
        let matrix_len = stream.read_value::<u16>().await? as usize;
        let matrix = match matrix_len {
            0 => None,
            _ => Some({
                let matrix_height = stream.read_value::<u32>().await? as usize;
                let matrix_width = stream.read_value::<u32>().await? as usize;
                let matrix_size = matrix_height * matrix_width;
                let mut matrix_data = Vec::with_capacity(matrix_size);
                for _ in 0..matrix_size {
                    matrix_data.push(stream.read_value().await?);
                }
                Array2D::from_row_major(&matrix_data, matrix_height, matrix_width).unwrap()
            }),
        };

        let segments = stream.read_value_min_version(4).await?.unwrap_or_default();
        let flags = stream.read_value_min_version(5).await?;

        Ok(ZoneData {
            id: u32::MAX,
            name,
            zone_type,
            leds_min,
            leds_max,
            leds_count,
            matrix,
            segments,
            flags,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use array2d::Array2D;
    use tokio_test::io::Builder;

    use crate::protocol::ReadableStream;
    use crate::protocol::data::{ZoneData, ZoneType};
    use crate::protocol::tests::setup;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&5_u16.to_le_bytes()) // name len
            .read(b"test\0") // name
            .read(&1_u32.to_le_bytes()) // type
            .read(&3_u32.to_le_bytes()) // leds_min
            .read(&18_u32.to_le_bytes()) // leds_max
            .read(&15_u32.to_le_bytes()) // leds_count
            .read(&0_u16.to_le_bytes()) // matrix_len
            .build();

        assert_eq!(
            stream.read_value::<ZoneData>().await?,
            ZoneData {
                name: "test".to_string(),
                zone_type: ZoneType::Linear,
                leds_min: 3,
                leds_max: 18,
                leds_count: 15,
                matrix: None,
                segments: vec![],
                flags: None,
                id: u32::MAX,
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
            .read(&1_u32.to_le_bytes()) // type
            .read(&3_u32.to_le_bytes()) // leds_min
            .read(&18_u32.to_le_bytes()) // leds_max
            .read(&15_u32.to_le_bytes()) // leds_count
            .read(&32_u16.to_le_bytes()) // matrix_len
            .read(&2_u32.to_le_bytes()) // matrix_height
            .read(&3_u32.to_le_bytes()) // matrix_width
            .read(&0_u32.to_le_bytes()) // matrix[0]
            .read(&1_u32.to_le_bytes()) // matrix[1]
            .read(&2_u32.to_le_bytes()) // matrix[2]
            .read(&3_u32.to_le_bytes()) // matrix[3]
            .read(&4_u32.to_le_bytes()) // matrix[4]
            .read(&5_u32.to_le_bytes()) // matrix[5]
            .build();

        assert_eq!(
            stream.read_value::<ZoneData>().await?,
            ZoneData {
                name: "test".to_string(),
                zone_type: ZoneType::Linear,
                leds_min: 3,
                leds_max: 18,
                leds_count: 15,
                matrix: Some(Array2D::from_rows(&[vec![0, 1, 2], vec![3, 4, 5]]).unwrap()),
                segments: vec![],
                flags: None,
                id: u32::MAX,
            }
        );

        Ok(())
    }
}
