use std::io::Read;
use std::mem::size_of;

use crate::protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};
use crate::{OpenRgbError, OpenRgbResult};

impl DeserFromBuf for String {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self>
    where
        Self: Sized {
        let len = buf.read_u16()? as usize;
        let mut bytes = vec![0u8; len];
        buf.read_exact(&mut bytes)?;
        bytes.pop(); // null byte?
        String::from_utf8(bytes).map_err(|e| {
            OpenRgbError::ProtocolError(format!("Failed decoding string as UTF-8: {}", e))
        })
    }
}

impl SerToBuf for String {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        self.as_str().serialize(buf)
    }
}

impl SerToBuf for &str {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u16(self.len() as u16 + 1); // +1 for null terminator
        buf.write_value(&RawString(self))?;
        Ok(())
    }
}


/// A raw string that does not include the length in its serialized form.
///
/// If the length is needed, serialize a `&str` or `String` instead.
#[doc(hidden)]
pub struct RawString<'a>(pub &'a str);

impl SerToBuf for RawString<'_> {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_slice(self.0.as_bytes());
        buf.write_u8(b'\0');
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::error::Error;

//     use tokio_test::io::Builder;

//     use crate::protocol::data::implement::string::RawString;
//     use crate::protocol::tests::setup;

//     #[tokio::test]
//     async fn test_read_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new()
//             .read(&5_u16.to_le_bytes())
//             .read(b"test\0")
//             .build();

//         assert_eq!(stream.read_value::<String>().await?, "test".to_string());

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new()
//             .write(&5_u16.to_le_bytes())
//             .write(b"test\0")
//             .build();

//         stream.write_value(&"test").await?;

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_raw_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().write(b"test\0").build();

//         stream.write_value(&RawString("test")).await?;

//         Ok(())
//     }
// }
