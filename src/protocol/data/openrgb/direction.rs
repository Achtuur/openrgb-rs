use std::mem::size_of;

use num_traits::FromPrimitive;

use crate::protocol::{ReadableStream, WritableStream};
use crate::protocol::{TryFromStream, Writable};
use crate::{OpenRgbError, OpenRgbResult};

/// Direction for [Mode](crate::data::Mode).
#[derive(Primitive, Eq, PartialEq, Debug, Copy, Clone, Default)]
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

impl Writable for Direction {
    fn size(&self) -> usize {
        size_of::<u32>()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        let num = *self as u32;
        stream.write_value(&num).await
    }
}

impl TryFromStream for Direction {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        stream.read_value().await.and_then(|id| {
            Direction::from_u32(id)
                .ok_or_else(|| OpenRgbError::ProtocolError(format!("unknown direction \"{}\"", id)))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::protocol::data::Direction;
    use tokio_test::io::Builder;

    use crate::protocol::tests::setup;
    use crate::protocol::{ReadableStream, WritableStream};

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&4_u32.to_le_bytes()).build();

        assert_eq!(
            stream.read_value::<Direction>().await?,
            Direction::Horizontal
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&4_u32.to_le_bytes()).build();

        stream.write_value(&Direction::Horizontal).await?;

        Ok(())
    }
}
