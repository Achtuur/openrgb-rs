use crate::protocol::{ReadableStream, TryFromStream};
use crate::OpenRgbError;

/// A single LED.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct LED {
    /// LED name.
    pub name: String,

    /// LED value.
    ///
    /// This is some internal flag, basically of no use to us
    pub value: u32,
}

impl TryFromStream for LED {
    async fn try_read(
        stream: &mut impl ReadableStream,
    ) -> Result<Self, OpenRgbError> {
        let name = stream.read_value().await?;
        let value = stream.read_value().await?;
        Ok(LED {
            name,
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::data::LED;
    use crate::protocol::ReadableStream;
    use crate::tests::setup;
    use crate::DEFAULT_PROTOCOL;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&5_u16.to_le_bytes())
            .read(b"test\0")
            .read(&45_u32.to_le_bytes())
            .build();

        assert_eq!(
            stream.read_value::<LED>(DEFAULT_PROTOCOL).await?,
            LED {
                name: "test".to_string(),
                value: 45
            }
        );

        Ok(())
    }
}
