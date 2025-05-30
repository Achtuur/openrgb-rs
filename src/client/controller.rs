use crate::{
    OpenRgbError, OpenRgbResult, ProtocolTcpStream,
    data::ModeData,
    protocol::{
        OpenRgbProtocol,
        data::{Color, ControllerData},
    },
};

use super::{zone, Zone};

pub struct Controller {
    id: u32,
    proto: OpenRgbProtocol<ProtocolTcpStream>,
    data: ControllerData,
}

impl Controller {
    pub fn new(id: u32, proto: OpenRgbProtocol<ProtocolTcpStream>, data: ControllerData) -> Self {
        Self { id, proto, data }
    }

    /// Connects to the OpenRGB server with a new client.
    ///
    /// This can be done to give each device its own connection.
    pub async fn connect_new_client(&mut self) -> OpenRgbResult<()> {
        let new_proto = self.proto.connect_clone().await?;
        new_proto.set_name(&self.data().name).await?;
        self.proto = new_proto;
        Ok(())
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
        let mode = self
            .get_mode_if_contains("direct")
            .or(self.get_mode_if_contains("custom"))
            .or(self.get_mode_if_contains("static"))
            .ok_or(OpenRgbError::ProtocolError(
                "No controllable mode found".to_string(),
            ))?;

        self.proto.update_mode(self.id, mode).await
    }

    pub async fn set_disabled_mode(&self) -> OpenRgbResult<()> {
        // order: "disabled", "off"
        let mode = self
            .get_mode_if_contains("disabled")
            .or(self.get_mode_if_contains("off"))
            .ok_or(OpenRgbError::ProtocolError(
                "No disabled mode found".to_string(),
            ))?;

        self.proto.update_mode(self.id, mode).await
    }

    fn get_mode_if_contains(&self, pat: &str) -> Option<&ModeData> {
        self.data()
            .modes
            .iter()
            .find(|m| m.name.to_lowercase().contains(pat))
    }

    pub fn get_zone<'a>(&'a self, zone_id: u32) -> OpenRgbResult<Zone<'a>> {
        let zone_data = self
            .data
            .zones
            .get(zone_id as usize)
            .ok_or(OpenRgbError::ProtocolError(format!(
                "zone {} not found",
                zone_id
            )))?;
        let zone = Zone::new(self.id, zone_id, &self.proto, zone_data);
        Ok(zone)
    }

    pub async fn update_led(&self, led: i32, color: Color) -> OpenRgbResult<()> {
        self.proto.update_led(self.id(), led, &color).await
    }

    pub async fn update_all_leds(&self, color: Color) -> OpenRgbResult<()> {
        let n_leds = self
            .data
            .zones
            .iter()
            .map(|zone| zone.leds_count)
            .sum::<u32>() as usize;
        let colors = vec![color; n_leds];
        self.update_leds(&colors).await?;
        Ok(())
    }

    pub async fn update_leds(&self, colors: &[Color]) -> OpenRgbResult<()> {
        self.proto.update_leds(self.id(), colors).await
    }

    pub async fn update_zone(&self, zone_id: u32, colors: &[Color]) -> OpenRgbResult<()> {
        self.get_zone(zone_id)?.update_leds(colors).await
    }

    /// Updates multiple zones with their respective colors.
    ///
    /// # Important
    ///
    /// The zone id's and colors MUST BE IN ORDER
    ///
    /// Pads the colors with black if the number of colors is less than the number of LEDs in the zone.
    pub async fn update_multiple_zones(&self, zone_colors: impl IntoIterator<Item = (u32, impl IntoIterator<Item = Color>)>) -> OpenRgbResult<()> {
        let colors = zone_colors
        .into_iter()
        .filter_map(|(z_id, c)| {
            let zone = self.get_zone(z_id).ok()?;
            Some(zone.zone_colors_from_iter(c))
        })
        .flatten()
        .collect::<Vec<_>>();
        self.update_leds(&colors).await
    }

    pub async fn disable_all_leds(&self) -> OpenRgbResult<()> {
        // match self.set_disabled_mode().await {
        //     Ok(_) => Ok(()),
        //     Err(e) => {
        //         // tracing::warn!(
        //         //     "Failed to set disabled mode for controller {}: {}",
        //         //     self.id, e
        //         // );
        //     }
        // }
        self.set_controllable_mode().await?;
        self.update_all_leds(Color {r: 0, g: 0, b: 0}).await
    }
}
