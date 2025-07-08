use flagset::FlagSet;

use crate::{
    data::{ModeData, ModeFlag}, protocol::{
        data::{Color, ControllerData}, OpenRgbProtocol
    }, OpenRgbError, OpenRgbResult, ProtocolTcpStream
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
        let mut mode = self
            .get_mode_if_contains("direct")
            .or(self.get_mode_if_contains("custom"))
            .or(self.get_mode_if_contains("static"))
            .ok_or(OpenRgbError::ProtocolError(
                "No controllable mode found".to_string(),
            ))?.clone();

        tracing::debug!(
            "Setting {} to {} mode",
            self.name(),
            mode.name
        );

        if mode.flags.contains(ModeFlag::HasBrightness) {
            mode.brightness.replace(100);
            mode.brightness_min.replace(100);
            mode.brightness_max.replace(100);
        }

        // just do both I guess
        self.proto.update_mode(self.id, &mode).await?;
        self.proto.save_mode(self.id, &mode).await
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
            .find(|m| m.name.to_ascii_lowercase().contains(pat))
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
        self.update_leds(colors).await?;
        Ok(())
    }

    pub async fn update_leds(&self, colors: impl IntoIterator<Item = Color>) -> OpenRgbResult<()> {
        let color_v = colors.into_iter().collect::<Vec<_>>();
        self.proto.update_leds(self.id(), color_v.as_slice()).await
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
        // for (z_id, colors) in zone_colors {
        //     let zone = self.get_zone(z_id)?;
        //     let colors: Vec<Color> = zone.zone_colors_from_iter(colors).collect();
        //     zone.update_leds(&colors).await?;
        // }
        // Ok(())

        let mut zone_id_iter = 0..self.data().zones.len();
        let colors = zone_colors
        .into_iter()
        .filter_map(|(z_id, colors)| {
            // add padding for all zones up to this zone
            let mut colors_up_til_this_zone = Vec::new();
            for id in zone_id_iter.by_ref() {
                if id as u32 == z_id {
                    break; // found the zone
                }
                let padding = vec![Color::default(); self.data().zones[id].leds_count as usize];
                colors_up_til_this_zone.extend(padding);
            }

            let zone = self.get_zone(z_id).ok()?;
            let c = zone.zone_colors_from_iter(colors);
            colors_up_til_this_zone.extend(c);
            Some(colors_up_til_this_zone)
        })
        .flatten();
        self.update_leds(colors).await
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
