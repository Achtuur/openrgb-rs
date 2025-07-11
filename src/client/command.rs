use std::collections::HashMap;

use crate::{client::group::{ControllerGroup, ControllerIndex}, Color, Controller, OpenRgbError, OpenRgbResult};

pub enum UpdateCommand {
    Controller {
        controller_id: usize,
        colors: Vec<Color>,
    },
    Zone {
        controller_id: usize,
        zone_id: usize,
        colors: Vec<Color>,
    },
    Segment {
        controller_id: usize,
        zone_id: usize,
        segment_id: usize,
        colors: Vec<Color>,
    },
    Single {
        controller_id: usize,
        led_id: usize,
        color: Color,
    }
}

pub struct UpdateLedCommandGroup<'a> {
    group: &'a ControllerGroup,
    commands: HashMap<usize, UpdateLedCommand<'a>>,
}

impl<'a> UpdateLedCommandGroup<'a> {
    pub(crate) fn new(group: &'a ControllerGroup) -> Self {
        let map = group.controllers()
            .iter()
            .map(|c| (c.id(), UpdateLedCommand::new(c)))
            .collect();
        Self {
            group,
            commands: map,
        }
    }

    pub async fn execute(self) -> OpenRgbResult<()> {
        for cmd in self.commands.into_values() {
            cmd.execute().await?;
        }
        Ok(())
    }

    fn get_controller_mut(&mut self, controller_id: impl ControllerIndex) -> OpenRgbResult<&mut UpdateLedCommand<'a>> {
        let c = self.group.get_controller(controller_id)?;
        self.commands.get_mut(&c.id())
            .ok_or(OpenRgbError::CommandError(format!(
                "Controller with id {} not found in group",
                c.id()
            )))
    }

    pub fn add_update_led(&mut self, controller_id: impl ControllerIndex, led_id: usize, color: Color) -> OpenRgbResult<()> {
        let cmd = self.get_controller_mut(controller_id)?;
        cmd.add_update_led(led_id, color)
    }

    pub fn add_update_controller_leds(&mut self, controller_id: impl ControllerIndex, colors: Vec<Color>) -> OpenRgbResult<()> {
        let cmd = self.get_controller_mut(controller_id)?;
        cmd.add_update_all_leds(colors)
    }

    pub fn add_update_zone(&mut self, controller_id: impl ControllerIndex, zone_id: usize, colors: Vec<Color>) -> OpenRgbResult<()> {
        let cmd = self.get_controller_mut(controller_id)?;
        cmd.add_update_zone(zone_id, colors)
    }

    pub fn add_update_segment(&mut self, controller_id: impl ControllerIndex, zone_id: usize, segment_id: usize, colors: Vec<Color>) -> OpenRgbResult<()> {
        let cmd = self.get_controller_mut(controller_id)?;
        cmd.add_update_segment(zone_id, segment_id, colors)
    }
}


#[derive(Debug)]
pub struct UpdateLedCommand<'a> {
    controller: &'a Controller,
    colors: Vec<Color>,
}

impl<'a> UpdateLedCommand<'a> {
    pub(crate) fn new(controller: &'a Controller) -> Self {
        Self {
            controller,
            colors: Vec::with_capacity(controller.num_leds()),
        }
    }

    pub(crate) fn into_colors(self) -> Vec<Color> {
        self.colors
    }

    pub(crate) fn colors(&self) -> &[Color] {
        &self.colors
    }

    pub async fn execute(self) -> OpenRgbResult<()> {
        self.controller.update_leds(self.colors).await?;
        // self.controller.sync_controller_data().await?;
        Ok(())
    }

    #[inline(always)]
    pub fn push_update_led(&mut self, led_id: usize, color: Color) -> OpenRgbResult<&mut Self> {
        self.add_update_led(led_id, color)?;
        Ok(self)
    }

    #[inline(always)]
    pub fn push_update_all_leds(&mut self, colors: Vec<Color>) -> OpenRgbResult<&mut Self> {
        self.add_update_all_leds(colors)?;
        Ok(self)
    }

    #[inline(always)]
    pub fn push_update_zone(&mut self, zone_id: usize, colors: Vec<Color>) -> OpenRgbResult<&mut Self> {
        self.add_update_zone(zone_id, colors)?;
        Ok(self)
    }

    #[inline(always)]
    pub fn push_update_segment(&mut self, zone_id: usize, segment_id: usize, colors: Vec<Color>) -> OpenRgbResult<&mut Self> {
        self.add_update_segment(zone_id, segment_id, colors)?;
        Ok(self)
    }

    pub fn add_update_led(&mut self, led_id: usize, color: Color) -> OpenRgbResult<()> {
        self.add_command(UpdateCommand::Single {
            controller_id: self.controller.id(),
            led_id,
            color
        })
    }

    pub fn add_update_all_leds(&mut self, colors: Vec<Color>) -> OpenRgbResult<()> {
        self.add_command(UpdateCommand::Controller {
            controller_id: self.controller.id(),
            colors
        })
    }

    pub fn add_update_zone(&mut self, zone_id: usize, colors: Vec<Color>) -> OpenRgbResult<()> {
        self.add_command(UpdateCommand::Zone {
            controller_id: self.controller.id(),
            zone_id,
            colors
        })
    }

    pub fn add_update_segment(&mut self, zone_id: usize, segment_id: usize, colors: Vec<Color>) -> OpenRgbResult<()> {
        self.add_command(UpdateCommand::Segment {
            controller_id: self.controller.id(),
            zone_id,
            segment_id,
            colors
        })
    }

    pub fn extend_with(&mut self, commands: impl IntoIterator<Item = UpdateCommand>) -> OpenRgbResult<&mut Self> {
        for cmd in commands {
            self.add_command(cmd)?;
        }
        Ok(self)
    }

    #[inline(always)]
    pub fn push_command(&mut self, cmd: UpdateCommand) -> OpenRgbResult<&mut Self> {
        self.add_command(cmd)?;
        Ok(self)
    }

    pub fn add_command(&mut self, cmd: UpdateCommand) -> OpenRgbResult<()> {
        match cmd {
            UpdateCommand::Controller { controller_id, colors } => {
                if colors.len() > self.controller.num_leds() {
                    tracing::warn!(
                        "Controller {} was given {} colors, while its length is {}. This might become a hard error in the future.",
                        self.controller.name(), colors.len(), self.controller.num_leds()
                    )
                    // return Err(OpenRgbError::CommandError(format!(
                    //     "Controller {} was given {} colors, while its length is {}. This might become a hard error in the future.",
                    //     self.controller.name(), self.controller.num_leds(), colors.len()
                    // )));
                }

                self.set_colors(0, &colors)?;
            }
            UpdateCommand::Zone { controller_id, zone_id, colors } => {
                let zone = self.controller.get_zone(zone_id)?;
                if colors.len() >= zone.num_leds() {
                    tracing::warn!(
                        "Zone {} for controller {} was given {} colors, while its length is {}. This might become a hard error in the future.",
                        zone_id, self.controller.name(), colors.len(), zone.num_leds()
                    )
                }

                let offset = self.controller.get_zone_led_offset(zone_id)?;
                let len = colors.len().min(zone.num_leds());
                self.set_colors(offset, &colors[..len])?;
            }
            UpdateCommand::Segment { controller_id, zone_id, segment_id, colors } => {
                let zone = self.controller.get_zone(zone_id)?;
                let seg = zone.get_segment(segment_id)?;
                if colors.len() >= seg.num_leds() {
                    tracing::warn!(
                        "Segment {} for zone {} in controller {} was given {} colors, while its length is {}. This might become a hard error in the future.",
                        seg.name(), zone_id, self.controller.name(), colors.len(), seg.num_leds()
                    )
                }

                let offset = zone.offset() + seg.offset();
                self.set_colors(offset, &colors)?;
            }
            UpdateCommand::Single { controller_id, led_id, color } => {
                if led_id >= self.controller.num_leds() {
                    tracing::warn!(
                        "LED id {} is out of bounds for controller {} with {} LEDs",
                        led_id, self.controller.name(), self.controller.num_leds()
                    );
                }
                self.set_colors(led_id, &[color])?;
            }
        }
        Ok(())
    }

    /// This is only called internally, so it is safe to assume that the colors are properly bounded
    fn set_colors(&mut self, offset: usize, colors: &[Color]) -> OpenRgbResult<()> {
        let len = offset + colors.len();
        if self.colors.len() < len {
            self.colors.resize(len, Color::default());
        }
        self.colors[offset..len].copy_from_slice(&colors);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_from_slice_without_alloc() {
        let mut vec = Vec::with_capacity(8);
        vec.resize(4, 0);
        vec[0..4].copy_from_slice(&[1, 2, 3, 4]);
        println!("vec: {0:?}", vec);
    }
}