use crate::protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};
use crate::OpenRgbResult;

impl<T: DeserFromBuf> DeserFromBuf for Vec<T> {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self>
    where
        Self: Sized {
        let len = buf.read_u16()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::deserialize(buf)?);
        }
        Ok(vec)
    }
}

impl<T: SerToBuf> SerToBuf for Vec<T> {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u16(self.len() as u16);
        for t in self {
            buf.write_value(t)?;
        }
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::error::Error;

//     use tokio_test::io::Builder;

//     use crate::protocol::tests::setup;

//     #[tokio::test]
//     async fn test_read_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new()
//             .read(&3_u16.to_le_bytes())
//             .read(&[37_u8, 54_u8, 126_u8])
//             .build();

//         assert_eq!(
//             stream.read_value::<Vec<u8>>().await?,
//             vec![37_u8, 54_u8, 126_u8]
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new()
//             .write(&3_u16.to_le_bytes())
//             .write(&[37_u8, 54_u8, 126_u8])
//             .build();

//         stream.write_value(&vec![37_u8, 54_u8, 126_u8]).await?;

//         Ok(())
//     }
// }
