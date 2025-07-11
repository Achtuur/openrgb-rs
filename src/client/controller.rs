use crate::{
    OpenRgbError, OpenRgbResult,
    client::command::UpdateLedCommand,
    data::{ModeData, ModeFlag},
    protocol::{
        OpenRgbProtocol,
        data::{Color, ControllerData},
    },
};

use super::Zone;

/// An RGBController, which represents a single RGB device that can be controlled.
///
/// # Example
/// todo
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

    pub(crate) fn proto(&self) -> &OpenRgbProtocol {
        &self.proto
    }

    /// Returns the ID of this controller.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Returns the name of this controller.
    pub fn name(&self) -> &str {
        &self.data.name
    }

    /// Returns the protocol version of this controller.
    pub fn data(&self) -> &ControllerData {
        &self.data
    }

    /// Returns the number of LEDs in this controller.
    pub fn num_leds(&self) -> usize {
        self.data.num_leds
    }

    /// Initialises a controller by setting it to a controllable mode.
    /// This function also changes the LEDs to a rainbow, so you can see if it worked.
    pub async fn init(&self) -> OpenRgbResult<()> {
        self.set_controllable_mode().await?;
        const RAINBOW_COLORS: [Color; 7] = [
            Color::new(255, 0, 0),   // Red
            Color::new(255, 127, 0), // Orange
            Color::new(255, 255, 0), // Yellow
            Color::new(0, 255, 0),   // Green
            Color::new(0, 0, 255),   // Blue
            Color::new(75, 0, 130),  // Indigo
            Color::new(148, 0, 211), // Violet
        ];
        let colors = (0..self.num_leds())
            .map(|i| i * RAINBOW_COLORS.len() / self.num_leds())
            .map(|i| RAINBOW_COLORS[i]);
        self.set_leds(colors).await?;
        Ok(())
    }

    /// Saves the current mode of this controller to the flash memory of the controller.
    ///
    /// # Important
    ///
    /// Using this frequently can cause wear on the flash memory, use this sparingly.
    pub async fn save_mode(&self) -> OpenRgbResult<()> {
        let Some(active_mode) = self.data.active_mode() else {
            return Err(OpenRgbError::CommandError(format!(
                "Controller {} has no active mode",
                self.name()
            )));
        };
        if !active_mode.flags.contains(ModeFlag::ManualSave) {
            return Err(OpenRgbError::CommandError(format!(
                "Controller {} mode {} cannot be saved",
                self.name(),
                active_mode.name
            )));
        }
        self.proto.save_mode(self.id as u32, active_mode).await
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
            ))?
            .clone();

        tracing::debug!("Setting {} to {} mode", self.name(), mode.name);

        if mode.flags.contains(ModeFlag::HasBrightness) {
            mode.brightness.replace(100);
            mode.brightness_min.replace(100);
            mode.brightness_max.replace(100);
        }

        // just do both I guess
        self.proto.update_mode(self.id as u32, &mode).await?;
        self.proto.save_mode(self.id as u32, &mode).await
    }

    fn get_mode_if_contains(&self, pat: &str) -> Option<&ModeData> {
        self.data()
            .modes
            .iter()
            .find(|m| m.name.to_ascii_lowercase().contains(pat))
    }

    /// Returns the zone with the given `zone_id`.
    pub fn get_zone<'a>(&'a self, zone_id: usize) -> OpenRgbResult<Zone<'a>> {
        if self.data.zones.get(zone_id).is_none() {
            return Err(OpenRgbError::CommandError(format!(
                "Zone {zone_id} not found for {}",
                self.name()
            )));
        }
        let zone = Zone::new(self, zone_id);
        Ok(zone)
    }

    /// Returns an iterator over all available zones in this controller.
    pub fn get_all_zones<'a>(&'a self) -> impl Iterator<Item = Zone<'a>> {
        self.data
            .zones
            .iter()
            .map(|z| Zone::new(self, z.id as usize))
    }

    /// Sets a single LED to the given `color`.
    ///
    /// When doing many writes in rapid succession, it is recommended to use the `cmd()` method instead.
    pub async fn set_led(&self, led: usize, color: Color) -> OpenRgbResult<()> {
        self.proto
            .update_led(self.id as u32, led as i32, &color)
            .await
    }

    /// Sets all LEDs of this controller to a given `color`.
    pub async fn set_all_leds(&self, color: Color) -> OpenRgbResult<()> {
        let colors = vec![color; self.num_leds()];
        self.set_leds(colors).await?;
        Ok(())
    }

    /// Sets the LEDs of this controller to the given `colors`.
    pub async fn set_leds(&self, colors: impl IntoIterator<Item = Color>) -> OpenRgbResult<()> {
        let color_v = colors.into_iter().collect::<Vec<_>>();
        self.proto
            .update_leds(self.id as u32, color_v.as_slice())
            .await
    }

    /// Sets the LEDs of a specific zone to the given `colors`.
    pub async fn set_zone_leds(
        &self,
        zone_id: usize,
        colors: impl IntoIterator<Item = Color>,
    ) -> OpenRgbResult<()> {
        let color_v = colors.into_iter().collect::<Vec<_>>();
        self.proto
            .update_zone_leds(self.id as u32, zone_id as u32, &color_v)
            .await
    }

    /// Clears all segments of this controller.
    pub async fn clear_segments(&self) -> OpenRgbResult<()> {
        self.proto.clear_segments(self.id as u32).await
    }

    /// Turns off all LEDs of this controller.
    pub async fn turn_off_leds(&self) -> OpenRgbResult<()> {
        self.set_controllable_mode().await?;
        self.set_all_leds(Color { r: 0, g: 0, b: 0 }).await
    }

    /// Creates an `UpdateLedCommand` for this controller.
    ///
    /// Controller LEDs can be updated in three ways:
    ///  * per led: `self.set_led()`
    ///  * per zone: `self.set_zone_leds()`
    ///  * all at once: `self.set_leds()`
    ///
    /// From my testing, the most efficient way is to always update all LEDs at once.
    /// The `UpdateLedCommand` API lets you build a command using updates to individual LEDs, zones or segments
    /// and then executes them as a single `set_led()` call.
    ///
    /// # Example
    /// ```no_run
    /// // let's say we have a controller with 5 LEDs
    /// let mut client = OpenRgbClient::connect().await?;
    /// let controller = client.get_controller(0).await?;
    ///
    /// // direct write
    /// controller.set_leds([Color::new(255, 0, 0); 5)]).await?;
    ///
    /// // equivalent with command
    /// let mut cmd = controller.cmd();
    /// cmd.add_update_led(0, Color::new(255, 0, 0))?;
    /// cmd.add_update_led(2, Color::new(255, 0, 0))?; // order doesn't matter
    /// cmd.add_update_led(4, Color::new(255, 0, 0))?;
    /// cmd.add_update_led(1, Color::new(255, 0, 0))?;
    /// cmd.add_update_led(5, Color::new(255, 0, 0))?;
    /// // this is just a single update
    /// cmd.execute().await?;
    /// ```
    ///
    /// This is especially useful for devices with multiple zones that should animate separately.
    pub fn cmd(&self) -> UpdateLedCommand<'_> {
        UpdateLedCommand::new(self)
    }

    pub(crate) fn get_zone_led_offset(&self, zone_id: usize) -> OpenRgbResult<usize> {
        if zone_id >= self.data.zones.len() {
            return Err(OpenRgbError::ProtocolError(format!(
                "zone {zone_id} not found in controller {}",
                self.id
            )));
        }

        let offset = self
            .data
            .zones
            .iter()
            .enumerate()
            .filter(|(idx, _)| *idx < zone_id)
            .map(|(_, z)| z.leds_count as usize)
            .sum::<usize>();
        Ok(offset)
    }

    /// Fetches controller data again. This updates the state of the controller data.
    ///
    /// Currently this has to be called manually.
    pub async fn sync_controller_data(&mut self) -> OpenRgbResult<()> {
        let data = self.proto.get_controller(self.id as u32).await?;
        self.data = data;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenRgbClient;

    use super::*;

    #[tokio::test]
    #[ignore = "can only test with openrgb running"]
    async fn test_update_leds() -> OpenRgbResult<()> {
        let client = OpenRgbClient::connect().await?;
        let controller = client.get_controller(0).await?;
        controller.set_controllable_mode().await?;
        controller.set_leds([Color::new(255, 0, 50); 96]).await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore = "can only test with openrgb running"]
    async fn test_cmd() -> OpenRgbResult<()> {
        let client = OpenRgbClient::connect().await?;
        let controller = client.get_controller(0).await?;
        controller.set_controllable_mode().await?;
        let mut cmd = controller.cmd();
        cmd.add_set_led(19, Color::new(255, 0, 255))?;
        cmd.add_set_zone_leds(0, vec![Color::new(255, 255, 0); 19])?;
        cmd.add_set_zone_leds(1, vec![Color::new(0, 255, 255); 75])?;
        // controller.execute_command(cmd).await?;
        cmd.execute().await?;
        Ok(())
    }
}
