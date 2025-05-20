use std::mem::size_of;

use num_traits::FromPrimitive;

use crate::data::{TryFromStream, Writable};
use crate::protocol::{ReadableStream, WritableStream};
use crate::{OpenRgbError, OpenRgbResult};
use crate::OpenRgbError::ProtocolError;

/// Direction for [Mode](crate::data::Mode).
#[derive(Primitive, Eq, PartialEq, Debug, Copy, Clone)]
pub enum Direction {
    /// Left direction.
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

impl Default for Direction {
    fn default() -> Self {
        Direction::Left
    }
}

impl Writable for Direction {
    fn size(&self, _protocol: u32) -> usize {
        size_of::<u32>()
    }

    async fn try_write(
        self,
        stream: &mut impl WritableStream,
        protocol: u32,
    ) -> OpenRgbResult<()> {
        stream.write_value(self as u32, protocol).await
    }
}

impl TryFromStream for Direction {
    async fn try_read(
        stream: &mut impl ReadableStream,
        protocol: u32,
    ) -> Result<Self, OpenRgbError> {
        stream.read_value(protocol).await.and_then(|id| {
            Direction::from_u32(id)
                .ok_or_else(|| ProtocolError(format!("unknown direction \"{}\"", id)))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::data::Direction;
    use tokio_test::io::Builder;

    use crate::protocol::{ReadableStream, WritableStream};
    use crate::tests::setup;
    use crate::DEFAULT_PROTOCOL;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().read(&4_u32.to_le_bytes()).build();

        assert_eq!(
            stream.read_value::<Direction>(DEFAULT_PROTOCOL).await?,
            Direction::Horizontal
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new().write(&4_u32.to_le_bytes()).build();

        stream
            .write_value(Direction::Horizontal, DEFAULT_PROTOCOL)
            .await?;

        Ok(())
    }
}
