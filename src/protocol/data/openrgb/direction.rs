
use crate::protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};
use crate::{impl_enum_discriminant, OpenRgbError, OpenRgbResult};

/// Direction for [Mode](crate::data::Mode).
#[derive(Eq, PartialEq, Debug, Copy, Clone, Default)]
pub enum Direction {
    /// Left direction.
    #[default]
    Left = 0,

    /// Right direction.
    Right = 1,

    /// Up direction.
    Up = 2,

    /// Down direction.
    Down = 3,

    /// Horizontal direction.
    Horizontal = 4,

    /// Vertical direction.
    Vertical = 5,
}

impl_enum_discriminant!(
    Direction,
    Left: 0,
    Right: 1,
    Up: 2,
    Down: 3,
    Horizontal: 4,
    Vertical: 5
);

impl DeserFromBuf for Direction {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let direction_raw = buf.read_u32()?;
        Direction::try_from(direction_raw)
    }
}

impl SerToBuf for Direction {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        let num = u32::from(self);
        buf.write_u32(num);
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::error::Error;

//     use crate::protocol::data::Direction;
//     use tokio_test::io::Builder;

//     use crate::protocol::tests::setup;

//     #[tokio::test]
//     async fn test_read_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().read(&4_u32.to_le_bytes()).build();

//         assert_eq!(
//             stream.read_value::<Direction>().await?,
//             Direction::Horizontal
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().write(&4_u32.to_le_bytes()).build();

//         stream.write_value(&Direction::Horizontal).await?;

//         Ok(())
//     }
// }
