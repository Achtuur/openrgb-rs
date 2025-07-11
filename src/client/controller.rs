
use crate::{
    client::command::UpdateLedCommand, data::{ModeData, ModeFlag}, protocol::{
        data::{Color, ControllerData}, OpenRgbProtocol
    }, OpenRgbError, OpenRgbResult
};

use super::Zone;

pub struct Controller {
    id: usize,
    proto: OpenRgbProtocol,
    data: ControllerData,
}

impl std::fmt::Debug for Controller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Controller")
            .field("id", &self.id)
            .field("name", &self.data.name)
            .field("num_leds", &self.data.num_leds)
            .field("modes", &self.data.modes.len())
            .finish()
    }
}

impl Controller {
    pub(crate) fn new(id: usize, proto: OpenRgbProtocol, data: ControllerData) -> Self {
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

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.data.name
    }

    pub fn data(&self) -> &ControllerData {
        &self.data
    }

    pub fn num_leds(&self) -> usize {
        self.data.num_leds
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
        self.proto.update_mode(self.id as u32, &mode).await?;
        self.proto.save_mode(self.id as u32, &mode).await
    }

    pub async fn set_disabled_mode(&self) -> OpenRgbResult<()> {
        // order: "disabled", "off"
        let mode = self
            .get_mode_if_contains("disabled")
            .or(self.get_mode_if_contains("off"))
            .ok_or(OpenRgbError::ProtocolError(
                "No disabled mode found".to_string(),
            ))?;

        self.proto.update_mode(self.id as u32, mode).await
    }

    fn get_mode_if_contains(&self, pat: &str) -> Option<&ModeData> {
        self.data()
            .modes
            .iter()
            .find(|m| m.name.to_ascii_lowercase().contains(pat))
    }

    pub fn get_zone<'a>(&'a self, zone_id: usize) -> OpenRgbResult<Zone<'a>> {
        if self.data.zones.get(zone_id).is_none() {
            return Err(OpenRgbError::CommandError(format!(
                "Zone {zone_id} not found for {}", self.name()
            )));
        }
        let zone = Zone::new(self, zone_id);
        Ok(zone)
    }

    pub async fn update_led(&self, led: i32, color: Color) -> OpenRgbResult<()> {
        self.proto.update_led(self.id as u32, led, &color).await
    }

    pub async fn update_all_leds(&self, color: Color) -> OpenRgbResult<()> {
        let colors = vec![color; self.num_leds()];
        self.update_leds(colors).await?;
        Ok(())
    }

    pub async fn update_leds(&self, colors: impl IntoIterator<Item = Color>) -> OpenRgbResult<()> {
        let color_v = colors.into_iter().collect::<Vec<_>>();
        self.proto.update_leds(self.id as u32, color_v.as_slice()).await
    }

    pub async fn update_zone(&self, zone_id: usize, colors: &[Color]) -> OpenRgbResult<()> {
        self.proto.update_zone_leds(self.id as u32, zone_id as u32, colors).await
    }

    /// Updates multiple zones with their respective colors.
    ///
    /// # Important
    ///
    /// The zone id's and colors MUST BE IN ORDER
    ///
    /// Pads the colors with black if the number of colors is less than the number of LEDs in the zone.
    pub async fn update_multiple_zones(&self, zone_colors: impl IntoIterator<Item = (usize, impl IntoIterator<Item = Color>)>) -> OpenRgbResult<()> {
        let mut zone_id_iter = 0..self.data().zones.len();
        let colors = zone_colors
        .into_iter()
        .filter_map(|(z_id, colors)| {
            // add padding for all zones up to this zone
            let mut colors_up_til_this_zone = Vec::new();
            for id in zone_id_iter.by_ref() {
                if id == z_id {
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
        self.set_controllable_mode().await?;
        self.update_all_leds(Color {r: 0, g: 0, b: 0}).await
    }

    pub fn cmd(&self) -> UpdateLedCommand<'_> {
        UpdateLedCommand::new(self)
    }

    pub async fn execute_command(&mut self, cmd: UpdateLedCommand<'_>) -> OpenRgbResult<()> {
        let colors = cmd.into_colors();
        self.proto.update_leds(self.id() as u32, &colors).await?;
        self.sync_controller_data().await?;
        Ok(())
    }


    pub(crate) fn get_zone_led_offset(&self, zone_id: usize) -> OpenRgbResult<usize> {
        if zone_id >= self.data.zones.len() {
            return Err(OpenRgbError::ProtocolError(format!(
                "zone {zone_id} not found in controller {}",
                self.id
            )));
        }

        let offset = self.data.zones.iter().enumerate()
        .filter(|(idx, _)| *idx < zone_id)
        .map(|(_, z)| z.leds_count as usize)
        .sum::<usize>();
        Ok(offset)
    }

    /// Fetches controller data again.
    ///
    /// This is needed to sync the colors, so we keep colors the same as much as possible.
    pub(crate) async fn sync_controller_data(&mut self) -> OpenRgbResult<()> {
        let data = self.proto.get_controller(self.id as u32).await?;
        self.data = data;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::{client::controller, OpenRgbClientWrapper};

    use super::*;

    #[tokio::test]
    async fn test_perf() -> OpenRgbResult<()> {
        let mut client = OpenRgbClientWrapper::connect().await?;
        const C: Color = Color { r: 255, g: 0, b: 0 };
        const N: usize = 100;

        let mut controller = client.get_controller(5).await?;
        let timer = std::time::Instant::now();
        for _ in 0..N {
            controller.update_led(0, C).await?;
            controller = client.get_controller(5).await?;
        }
        println!("{:?} ms", 1000.0 * timer.elapsed().as_secs_f64() / N as f64);
        // let timer = std::time::Instant::now();
        // for _ in 0..N {
        //     controller.update_led(0, C).await?;
        // }
        // println!("single {:?}", timer.elapsed().as_secs_f64() / N as f64);

        // let timer = std::time::Instant::now();
        // for _ in 0..N {
        //     controller.update_leds([C; 20]).await?;
        // }
        // println!("all: {:?}", timer.elapsed());

        // let timer = std::time::Instant::now();
        // for _ in 0..N {
        //     controller.update_zone(0, &[C; 20]).await?;
        // }
        // println!("zone: {:?}", timer.elapsed());


        Ok(())
    }

    #[tokio::test]
    #[ignore = "Requires real OpenRGB server and human observer"]
    async fn test_update_leds() -> OpenRgbResult<()> {
        let mut client = OpenRgbClientWrapper::connect().await?;
        let mut controller = client.get_controller(5).await?;
        controller.set_controllable_mode().await?;
        controller.update_leds([Color::new(255, 0, 50); 96]).await?;
        Ok(())
    }

    #[tokio::test]
    #[traced_test(INFO)]
    async fn test_cmd() -> OpenRgbResult<()> {
        let mut client = OpenRgbClientWrapper::connect().await?;
        let mut controller = client.get_controller(5).await?;
        controller.set_controllable_mode().await?;
        let mut cmd = controller.cmd();
        cmd.add_update_led(19, Color::new(255, 0, 255))?;
        cmd.add_update_zone(0, vec![Color::new(255, 255, 0); 19])?;
        cmd.add_update_zone(1, vec![Color::new(0, 255, 255); 75])?;
        println!("cmd: {0:?}", cmd);
        cmd.execute().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_offset() -> OpenRgbResult<()> {
        let mut client = OpenRgbClientWrapper::connect().await?;
        let controller = client.get_controller(5).await?;
        let offset = controller.get_zone_led_offset(0)?;
        assert_eq!(offset, 0);
        let offset = controller.get_zone_led_offset(1)?;
        assert_eq!(offset, 20);
        let offset = controller.get_zone_led_offset(2)?;
        assert_eq!(offset, 20 + 70);
        Ok(())
    }
}