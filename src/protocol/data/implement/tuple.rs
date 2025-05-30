use crate::OpenRgbResult;
use crate::protocol::{ReadableStream, TryFromStream, Writable, WritableStream};


macro_rules! impl_tuple {
    ($($idx:tt $t:tt),+) => {
        impl<$($t: Writable),+> Writable for ($($t,)+) {
            fn size(&self) -> usize {
                0 $(+ self.$idx.size())+
            }

            async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
                let mut n = 0;
                $(
                    n += stream.write_value(&self.$idx).await?;
                )+
                Ok(n)
            }
        }

        impl<$($t: TryFromStream),+> TryFromStream for ($($t,)+) {
            async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
                Ok((
                    $(
                        stream.read_value::<$t>().await?,
                    )+
                ))
            }
        }
    }
}

impl_tuple!(0 A);
impl_tuple!(0 A, 1 B);
impl_tuple!(0 A, 1 B, 2 C);
impl_tuple!(0 A, 1 B, 2 C, 3 D);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E);



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

        let mut stream = Builder::new()
            .read(&37_u8.to_le_bytes())
            .read(&1337_u32.to_le_bytes())
            .read(&(-1337_i32).to_le_bytes())
            .read(&4_u32.to_le_bytes())
            .build();

        assert_eq!(
            stream.read_value::<(u8, u32, i32, DeviceType)>().await?,
            (37, 1337, -1337, DeviceType::LEDStrip)
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(&37_u8.to_le_bytes())
            .write(&1337_u32.to_le_bytes())
            .write(&(-1337_i32).to_le_bytes())
            .write(&4_u32.to_le_bytes())
            .build();

        stream
            .write_value(&(37_u8, 1337_u32, (-1337_i32), DeviceType::LEDStrip))
            .await?;

        Ok(())
    }
}
