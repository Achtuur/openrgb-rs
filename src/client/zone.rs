use crate::{data::ZoneData, Color, OpenRgbError, OpenRgbProtocol, OpenRgbResult, OpenRgbStream, ProtocolTcpStream};

pub struct Zone {
    zone_id: u32,
    controller_id: u32,
    proto: OpenRgbProtocol<ProtocolTcpStream>,
    data: ZoneData,
}

impl Zone {
    pub fn new(controller_id: u32, zone_id: u32, proto: OpenRgbProtocol<ProtocolTcpStream>, data: ZoneData) -> Self {
        Self {
            zone_id,
            controller_id,
            proto,
            data,
        }
    }

    pub fn controller_id(&self) -> u32 {
        self.controller_id
    }

    pub fn zone_id(&self) -> u32 {
        self.zone_id
    }

    pub async fn update_leds(&self, colors: &[Color]) -> OpenRgbResult<()> {
        if colors.len() != self.data.leds_count as usize {
            return Err(OpenRgbError::ProtocolError(format!(
                "Invalid number of colors: expected {}, got {}",
                self.data.leds_count, colors.len()
            )));
        }

        self.proto.update_zone_leds(self.controller_id(), self.zone_id(), colors).await
    }

    pub async fn update_leds_uniform(&self, color: Color) -> OpenRgbResult<()> {
        let colors = vec![color; self.data.leds_count as usize];
        self.update_leds(&colors).await
    }
}
