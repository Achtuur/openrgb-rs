use crate::OpenRgbResult;
use crate::protocol::data::{Color, DeviceType, LED, ModeData, ZoneData};
use crate::protocol::{ReadableStream, TryFromStream};

/// RGB controller.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_controller_data) for more information.
#[derive(Debug, Eq, PartialEq)]
pub struct ControllerData {
    /// Controller type.
    pub device_type: DeviceType,

    /// Controller name.
    pub name: String,

    /// Controller vendor.
    pub vendor: String,

    /// Controller description.
    pub description: String,

    /// Controller version.
    pub version: String,

    /// Controller serial.
    pub serial: String,

    /// Controller location.
    pub location: String,

    /// Controller active mode index.
    pub active_mode: i32,

    /// Controller modes.
    pub modes: Vec<ModeData>,

    /// Controller zones.
    pub zones: Vec<ZoneData>,

    /// Controller LEDs.
    pub leds: Vec<LED>,

    /// Controller colors.
    pub colors: Vec<Color>,

    /// Alternate names for LEDs (?)
    ///
    /// Minimum protocol version: 5
    pub led_alt_names: Vec<String>,

    /// flags
    ///
    /// Minimum protocol version: 5
    pub flags: Option<u32>,

    /// not in protocol, but given by the request used to get this controller
    pub id: u32,
}

impl TryFromStream for ControllerData {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        let _data_size = stream.read_value::<u32>().await?;
        let device_type = stream.read_value().await?;
        let name = stream.read_value().await?;
        let vendor = stream.read_value().await?;
        let description = stream.read_value().await?;
        let version = stream.read_value().await?;
        let serial = stream.read_value().await?;
        let location = stream.read_value().await?;
        let num_modes = stream.read_value::<u16>().await?;
        let active_mode = stream.read_value().await?;
        let mut modes = Vec::with_capacity(num_modes as usize);
        for idx in 0..num_modes {
            let mut mode: ModeData = stream.read_value().await?;
            mode.index = idx as u32;
            modes.push(mode);
        }

        let mut zones: Vec<ZoneData> = stream.read_value().await?;
        for (idx, zone) in zones.iter_mut().enumerate() {
            zone.id = idx as u32;
        }

        let leds = stream.read_value().await?;
        let colors = stream.read_value().await?;
        let led_alt_names = stream.read_value_min_version(5).await?.unwrap_or_default();
        let flags = stream.read_value_min_version(5).await?;

        Ok(ControllerData {
            device_type,
            name,
            vendor,
            description,
            version,
            serial,
            location,
            active_mode,
            modes,
            zones,
            leds,
            colors,
            led_alt_names,
            flags,
            id: u32::MAX,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use ModeFlag::*;

    use crate::protocol::ReadableStream;
    use crate::protocol::data::{
        Color, ColorMode, ControllerData, DeviceType, ModeData, ModeFlag, ZoneData, ZoneType,
    };
    use crate::protocol::tests::setup;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&760_u32.to_le_bytes())
            .read(&[
                3, 0, 0, 0, 18, 0, 84, 104, 101, 114, 109, 97, 108, 116, 97, 107, 101, 32, 82, 105,
                105, 110, 103, 0, 12, 0, 84, 104, 101, 114, 109, 97, 108, 116, 97, 107, 101, 0, 25,
                0, 84, 104, 101, 114, 109, 97, 108, 116, 97, 107, 101, 32, 82, 105, 105, 110, 103,
                32, 68, 101, 118, 105, 99, 101, 0, 1, 0, 0, 1, 0, 0, 19, 0, 72, 73, 68, 58, 32, 47,
                100, 101, 118, 47, 104, 105, 100, 114, 97, 119, 49, 48, 0, 8, 0, 0, 0, 0, 0, 7, 0,
                68, 105, 114, 101, 99, 116, 0, 24, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                0, 0, 0, 0, 0, 7, 0, 83, 116, 97, 116, 105, 99, 0, 25, 0, 0, 0, 64, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 5, 0, 70, 108, 111, 119, 0, 0, 0,
                0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 83, 112, 101, 99,
                116, 114, 117, 109, 0, 4, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 7, 0, 82, 105, 112, 112, 108, 101, 0, 8, 0, 0, 0, 33, 0, 0, 0, 3, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 1, 0, 0, 0, 0, 0, 6, 0, 66, 108, 105, 110, 107, 0, 12, 0, 0, 0, 33, 0, 0,
                0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 6, 0, 80, 117, 108, 115, 101, 0, 16,
                0, 0, 0, 33, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 5, 0, 87, 97,
                118, 101, 0, 20, 0, 0, 0, 33, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 5,
                0, 16, 0, 82, 105, 105, 110, 103, 32, 67, 104, 97, 110, 110, 101, 108, 32, 49, 0,
                1, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 82, 105, 105, 110,
                103, 32, 67, 104, 97, 110, 110, 101, 108, 32, 50, 0, 1, 0, 0, 0, 0, 0, 0, 0, 20, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 82, 105, 105, 110, 103, 32, 67, 104, 97, 110, 110,
                101, 108, 32, 51, 0, 1, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0,
                82, 105, 105, 110, 103, 32, 67, 104, 97, 110, 110, 101, 108, 32, 52, 0, 1, 0, 0, 0,
                0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 82, 105, 105, 110, 103, 32, 67,
                104, 97, 110, 110, 101, 108, 32, 53, 0, 1, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0,
            ])
            .build();

        assert_eq!(
            stream.read_value::<ControllerData>().await?,
            ControllerData {
                id: 0,
                led_alt_names: Vec::new(),
                flags: None,
                device_type: DeviceType::Cooler,
                name: "Thermaltake Riing".to_string(),
                vendor: "Thermaltake".to_string(),
                description: "Thermaltake Riing Device".to_string(),
                version: "".to_string(),
                serial: "".to_string(),
                location: "HID: /dev/hidraw10".to_string(),
                active_mode: 0,
                modes: vec![
                    ModeData {
                        protocol_version: 0,
                        index: u32::MAX,
                        name: "Direct".to_string(),
                        value: 24,
                        flags: HasPerLEDColor.into(),
                        speed_min: None,
                        speed_max: None,
                        brightness_min: None,
                        brightness_max: None,
                        colors_min: None,
                        colors_max: None,
                        speed: None,
                        brightness: None,
                        direction: None,
                        color_mode: Some(ColorMode::PerLED),
                        colors: vec![],
                    },
                    ModeData {
                        protocol_version: 0,
                        index: u32::MAX,
                        name: "Static".to_string(),
                        value: 25,
                        flags: HasModeSpecificColor.into(),
                        speed_min: None,
                        speed_max: None,
                        brightness_min: None,
                        brightness_max: None,
                        colors_min: Some(1),
                        colors_max: Some(1),
                        speed: None,
                        brightness: None,
                        direction: None,
                        color_mode: Some(ColorMode::ModeSpecific),
                        colors: vec![Color { r: 0, g: 0, b: 0 }],
                    },
                    ModeData {
                        protocol_version: 0,
                        index: u32::MAX,
                        name: "Flow".to_string(),
                        value: 0,
                        flags: HasSpeed.into(),
                        speed_min: Some(3),
                        speed_max: Some(0),
                        brightness_min: None,
                        brightness_max: None,
                        colors_min: None,
                        colors_max: None,
                        speed: Some(2),
                        brightness: None,
                        direction: None,
                        color_mode: Some(ColorMode::None),
                        colors: vec![],
                    },
                    ModeData {
                        protocol_version: 0,
                        index: u32::MAX,
                        name: "Spectrum".to_string(),
                        value: 4,
                        flags: HasSpeed.into(),
                        speed_min: Some(3),
                        speed_max: Some(0),
                        brightness_min: None,
                        brightness_max: None,
                        colors_min: None,
                        colors_max: None,
                        speed: Some(2),
                        brightness: None,
                        direction: None,
                        color_mode: Some(ColorMode::None),
                        colors: vec![],
                    },
                    ModeData {
                        protocol_version: 0,
                        index: u32::MAX,
                        name: "Ripple".to_string(),
                        value: 8,
                        flags: HasSpeed | HasPerLEDColor,
                        speed_min: Some(3),
                        speed_max: Some(0),
                        brightness_min: None,
                        brightness_max: None,
                        colors_min: None,
                        colors_max: None,
                        speed: Some(2),
                        brightness: None,
                        direction: None,
                        color_mode: Some(ColorMode::PerLED),
                        colors: vec![],
                    },
                    ModeData {
                        protocol_version: 0,
                        index: u32::MAX,
                        name: "Blink".to_string(),
                        value: 12,
                        flags: HasSpeed | HasPerLEDColor,
                        speed_min: Some(3),
                        speed_max: Some(0),
                        brightness_min: None,
                        brightness_max: None,
                        colors_min: None,
                        colors_max: None,
                        speed: Some(2),
                        brightness: None,
                        direction: None,
                        color_mode: Some(ColorMode::PerLED),
                        colors: vec![],
                    },
                    ModeData {
                        protocol_version: 0,
                        index: u32::MAX,
                        name: "Pulse".to_string(),
                        value: 16,
                        flags: HasSpeed | HasPerLEDColor,
                        speed_min: Some(3),
                        speed_max: Some(0),
                        brightness_min: None,
                        brightness_max: None,
                        colors_min: None,
                        colors_max: None,
                        speed: Some(2),
                        brightness: None,
                        direction: None,
                        color_mode: Some(ColorMode::PerLED),
                        colors: vec![],
                    },
                    ModeData {
                        protocol_version: 0,
                        index: u32::MAX,
                        name: "Wave".to_string(),
                        value: 20,
                        flags: HasSpeed | HasPerLEDColor,
                        speed_min: Some(3),
                        speed_max: Some(0),
                        brightness_min: None,
                        brightness_max: None,
                        colors_min: None,
                        colors_max: None,
                        speed: Some(2),
                        brightness: None,
                        direction: None,
                        color_mode: Some(ColorMode::PerLED),
                        colors: vec![],
                    },
                ],
                zones: vec![
                    ZoneData {
                        name: "Riing Channel 1".to_string(),
                        zone_type: ZoneType::Linear,
                        leds_min: 0,
                        leds_max: 20,
                        leds_count: 0,
                        matrix: None,
                        segments: Vec::new(),
                        flags: None,
                        id: 0,
                    },
                    ZoneData {
                        name: "Riing Channel 2".to_string(),
                        zone_type: ZoneType::Linear,
                        leds_min: 0,
                        leds_max: 20,
                        leds_count: 0,
                        matrix: None,
                        segments: Vec::new(),
                        flags: None,
                        id: 0,
                    },
                    ZoneData {
                        name: "Riing Channel 3".to_string(),
                        zone_type: ZoneType::Linear,
                        leds_min: 0,
                        leds_max: 20,
                        leds_count: 0,
                        matrix: None,
                        segments: Vec::new(),
                        flags: None,
                        id: 0,
                    },
                    ZoneData {
                        name: "Riing Channel 4".to_string(),
                        zone_type: ZoneType::Linear,
                        leds_min: 0,
                        leds_max: 20,
                        leds_count: 0,
                        matrix: None,
                        segments: Vec::new(),
                        flags: None,
                        id: 0,
                    },
                    ZoneData {
                        name: "Riing Channel 5".to_string(),
                        zone_type: ZoneType::Linear,
                        leds_min: 0,
                        leds_max: 20,
                        leds_count: 0,
                        matrix: None,
                        segments: Vec::new(),
                        flags: None,
                        id: 0,
                    },
                ],
                leds: vec![],
                colors: vec![],
            }
        );

        Ok(())
    }
}
