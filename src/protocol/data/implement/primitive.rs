use std::mem::size_of;

use crate::protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};
use crate::{OpenRgbError, OpenRgbResult};


impl DeserFromBuf for () {
    fn deserialize(_buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        Ok(())
    }
}

impl SerToBuf for () {
    fn serialize(&self, _buf: &mut WriteMessage) -> OpenRgbResult<()> {
        Ok(())
    }
}

impl DeserFromBuf for u8 {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        buf.read_u8()
    }
}

impl SerToBuf for u8 {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u8(*self);
        Ok(())
    }
}

impl DeserFromBuf for u16 {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        buf.read_u16()
    }
}

impl SerToBuf for u16 {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u16(*self);
        Ok(())
    }
}

impl DeserFromBuf for u32 {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        buf.read_u32()
    }
}

impl SerToBuf for u32 {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u32(*self);
        Ok(())
    }
}

impl DeserFromBuf for i32 {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let x = buf.read_u32()?;
        Ok(x as i32)
    }
}

impl SerToBuf for i32 {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u32(*self as u32);
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::error::Error;

//     use tokio_test::io::Builder;

//     use crate::protocol::tests::setup;

//     #[tokio::test]
//     async fn test_read_void_001() -> Result<(), Box<dyn Error>> {
//         setup()?;
//         let stream = Builder::new().build();
//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_void_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().build();

//         stream.write_value(&()).await?;

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_read_u8_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().read(&[37_u8]).build();

//         assert_eq!(stream.read_value::<u8>().await?, 37);

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_u8_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().write(&[37_u8]).build();

//         stream.write_value(&37_u8).await?;

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_read_u16_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().read(&37_u16.to_le_bytes()).build();

//         assert_eq!(stream.read_value::<u16>().await?, 37);

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_u16_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().write(&37_u16.to_le_bytes()).build();

//         stream.write_value(&37_u16).await?;

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_read_u32_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().read(&185851_u32.to_le_bytes()).build();

//         assert_eq!(stream.read_value::<u32>().await?, 185851);

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_u32_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().write(&185851_u32.to_le_bytes()).build();

//         stream.write_value(&185851_u32).await?;

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_read_i32_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().read(&(-185851_i32).to_le_bytes()).build();

//         assert_eq!(stream.read_value::<i32>().await?, -185851_i32);

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_i32_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().write(&(-185851_i32).to_le_bytes()).build();

//         stream.write_value(&-185851_i32).await?;

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_read_usize_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().read(&185851_u32.to_le_bytes()).build();

//         assert_eq!(stream.read_value::<usize>().await?, 185851_usize);

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_usize_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().write(&185851_u32.to_le_bytes()).build();

//         stream.write_value(&185851_usize).await?;

//         Ok(())
//     }
// }
