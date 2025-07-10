
use crate::protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};
use crate::OpenRgbError::ProtocolError;
use crate::{impl_enum_discriminant, OpenRgbResult};


/// OpenRGB protocol packet ID.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#packet-ids) for more information.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum PacketId {
    /// Request RGBController device count from server.
    RequestControllerCount = 0,

    /// Request RGBController data block.
    RequestControllerData = 1,

    /// Request OpenRGB SDK protocol version from server.
    RequestProtocolVersion = 40,

    /// Send client name string to server.
    SetClientName = 50,

    /// Indicate to clients that device list has updated.
    DeviceListUpdated = 100,

    /// Request profile list.
    RequestProfileList = 150,

    /// Save current configuration in a new profile.
    RequestSaveProfile = 151,

    /// Load a given profile.
    RequestLoadProfile = 152,

    /// Delete a given profile.
    RequestDeleteProfile = 153,

    /// RGBController::ResizeZone().
    RGBControllerResizeZone = 1000,

    /// RGBController::ClearSegments().
    RgbControllerClearSegments = 1001,

    /// RGBController::AddSegment().
    RGBControllerAddSegment = 1002,

    /// RGBController::UpdateLEDs().
    RGBControllerUpdateLeds = 1050,

    /// RGBController::UpdateZoneLEDs().
    RGBControllerUpdateZoneLeds = 1051,

    /// RGBController::UpdateSingleLED().
    RGBControllerUpdateSingleLed = 1052,

    /// RGBController::SetCustomMode().
    RGBControllerSetCustomMode = 1100,

    /// RGBController::UpdateMode().
    RGBControllerUpdateMode = 1101,

    /// RGBController::SaveMode().
    RGBControllerSaveMode = 1102,
}

impl_enum_discriminant!(
    PacketId,
    RequestControllerCount: 0,
    RequestControllerData: 1,
    RequestProtocolVersion: 40,
    SetClientName: 50,
    DeviceListUpdated: 100,
    RequestProfileList: 150,
    RequestSaveProfile: 151,
    RequestLoadProfile: 152,
    RequestDeleteProfile: 153,
    RGBControllerResizeZone: 1000,
    RgbControllerClearSegments: 1001,
    RGBControllerAddSegment: 1002,
    RGBControllerUpdateLeds: 1050,
    RGBControllerUpdateZoneLeds: 1051,
    RGBControllerUpdateSingleLed: 1052,
    RGBControllerSetCustomMode: 1100,
    RGBControllerUpdateMode: 1101,
    RGBControllerSaveMode: 1102
);

impl DeserFromBuf for PacketId {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let packet_id_raw = buf.read_u32()?;
        PacketId::try_from(packet_id_raw)
    }
}

impl SerToBuf for PacketId {
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

//     use crate::protocol::{PacketId, tests::setup};

//     #[test]
//     fn test_convert_to_u32() {
//         assert_eq!(PacketId::DeviceListUpdated.to_u32(), Some(100));
//     }

//     #[test]
//     fn test_convert_from_u32() {
//         assert_eq!(PacketId::from_u32(100), Some(PacketId::DeviceListUpdated))
//     }

//     #[tokio::test]
//     async fn test_read_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().read(&1101_u32.to_le_bytes()).build();

//         assert_eq!(
//             stream.read_value::<PacketId>().await?,
//             PacketId::RGBControllerUpdateMode
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_write_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new().write(&1101_u32.to_le_bytes()).build();

//         stream
//             .write_value(&PacketId::RGBControllerUpdateMode)
//             .await?;

//         Ok(())
//     }
// }
