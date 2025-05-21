use std::mem::size_of;

use crate::protocol::{ReadableStream, TryFromStream, Writable, WritableStream};
use crate::{OpenRgbError, OpenRgbResult};

impl<T: Writable> Writable for Vec<T> {
    fn size(&self) -> usize {
        self.as_slice().size()
        // size_of::<u16>() // vec is preceded by its length
        // + self.iter().map(|e| e.size()).sum::<usize>()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<()> {
        self.as_slice().try_write(stream).await?;
        Ok(())
    }
}

impl<T: TryFromStream> TryFromStream for Vec<T> {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        let len = stream.read_value::<u16>().await? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(stream.read_value().await?);
        }
        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::protocol::tests::setup;
    use crate::protocol::{ReadableStream, WritableStream};

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&3_u16.to_le_bytes())
            .read(&[37_u8, 54_u8, 126_u8])
            .build();

        assert_eq!(
            stream.read_value::<Vec<u8>>().await?,
            vec![37_u8, 54_u8, 126_u8]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(&3_u16.to_le_bytes())
            .write(&[37_u8, 54_u8, 126_u8])
            .build();

        stream.write_value(&vec![37_u8, 54_u8, 126_u8]).await?;

        Ok(())
    }
}
