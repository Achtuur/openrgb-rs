use std::sync::Arc;

use crate::{data::ModeData, protocol::{data::{Color, ControllerData}, OpenRgbProtocol, OpenRgbStream}, OpenRgbError, OpenRgbResult, ProtocolTcpStream};

use super::Zone;

pub struct Controller {
    id: u32,
    proto: OpenRgbProtocol<ProtocolTcpStream>,
    data: ControllerData,
}

impl Controller {
    pub fn new(id: u32, proto: OpenRgbProtocol<ProtocolTcpStream>, data: ControllerData) -> Self {
        Self {
            id,
            proto,
            data,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.data.name
    }

    pub fn data(&self) -> &ControllerData {
        &self.data
    }

    /// Sets this controller to a controllable mode.
    pub async fn set_controllable_mode(&self) -> OpenRgbResult<()> {
        // order: "direct", "custom", "static"
        let mode = self.get_mode_if_contains("direct")
        .or(self.get_mode_if_contains("custom"))
        .or(self.get_mode_if_contains("static"))
        .ok_or(OpenRgbError::ProtocolError("No controllable mode found".to_string()))?;

        self.proto.update_mode(self.id, mode).await
    }

    fn get_mode_if_contains(&self, pat: &str) -> Option<&ModeData> {
        self.data().modes.iter().find(|m| m.name.to_lowercase().contains(pat))
    }

    pub fn get_zone(&self, zone_id: u32) -> OpenRgbResult<Zone> {
        let zone_data = self.data.zones.get(zone_id as usize)
            .ok_or(OpenRgbError::ProtocolError(format!("zone {} not found", zone_id)))?;
        let zone = Zone::new(self.id, zone_id, self.proto.clone(), zone_data.clone());
        Ok(zone)
    }

    pub async fn update_led(&self, led: i32, color: Color) -> OpenRgbResult<()> {
        self.proto
            .update_led(self.id(), led, color)
            .await
    }

    pub async fn update_all_leds(&self, color: Color) -> OpenRgbResult<()> {
        let n_leds = self.data.zones.iter().map(|zone| zone.leds_count).sum::<u32>() as usize;
        let colors = vec![color; n_leds];
        self.update_leds(&colors).await?;
        Ok(())
    }

    pub async fn update_leds(&self, colors: &[Color]) -> OpenRgbResult<()> {
        self.proto
            .update_leds(self.id(), colors)
            .await
    }

    pub async fn update_zone(&self, zone_id: u32, color: Color) -> OpenRgbResult<()> {
        let zone = self.get_zone(zone_id)?;
        zone.update_leds_uniform(color).await
    }
}