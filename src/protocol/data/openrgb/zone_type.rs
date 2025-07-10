
use crate::protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};
use crate::{impl_enum_discriminant, OpenRgbError, OpenRgbResult};

/// RGB controller [Zone](crate::data::Zone) type.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#zone-data) for more information.
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum ZoneType {
    /// Single zone.
    Single = 0,

    /// Linear zone.
    Linear = 1,

    /// Matrix zone.
    Matrix = 2,
}

impl_enum_discriminant!(ZoneType, Single: 0, Linear: 1, Matrix: 2);

impl DeserFromBuf for ZoneType {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let zone_type = buf.read_u32()?;
        ZoneType::try_from(zone_type)
            .map_err(|_| OpenRgbError::ProtocolError(format!("unknown zone type \"{}\"", zone_type)))
        // let zone_type_raw = buf.read_u32()?;
        // ZoneType::from_u32(zone_type_raw)
        //         .ok_or_else(|| OpenRgbError::ProtocolError(format!("unknown zone type \"{}\"", zone_type_raw)))
    }
}

impl SerToBuf for ZoneType {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        let num = u32::from(self);
        buf.write_u32(num);
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::error::Error;

//     use tokio_test::io::Builder;

//     use crate::protocol::data::ZoneType;
//     use crate::protocol::tests::setup;

//     #[tokio::test]
//     async fn test_read_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().read(&1_u32.to_le_bytes()).build();

//         assert_eq!(stream.read_value::<ZoneType>().await?, ZoneType::Linear);

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().write(&1_u32.to_le_bytes()).build();

//         stream.write_value(&ZoneType::Linear).await?;

//         Ok(())
//     }
// }
