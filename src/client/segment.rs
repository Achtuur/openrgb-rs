use crate::{client::command::UpdateCommand, data::SegmentData, Color, OpenRgbResult, Zone};

/// A segment in a zone, which can contain multiple LEDs.
pub struct Segment<'z> {
    zone: &'z Zone<'z>,
    segment_id: usize,
}

impl<'z> Segment<'z> {
    pub(crate) fn new(zone: &'z Zone<'z>, segment_id: usize) -> Self {
        Self { zone, segment_id }
    }

    /// Returns the ID of this segment.
    pub fn id(&self) -> usize {
        self.segment_id
    }

    /// Returns the ID of the the controller this segment's zone belongs to.
    pub fn controller_id(&self) -> usize {
        self.zone.controller_id()
    }

    /// Returns the ID of the zone this segment belongs to.
    pub fn zone_id(&self) -> usize {
        self.zone.zone_id()
    }

    /// Returns the name of this segment.
    pub fn name(&self) -> &str {
        self.data().name()
    }

    /// Returns the `SegmentData` for this segment.
    pub fn data(&self) -> &SegmentData {
        self.zone.data()
        .segments
        .value()
        .expect("Segment struct created with protocol version < 4")
        .get(self.segment_id)
        .expect("Segment data not found")
    }

    /// Returns the number of LEDs in this segment.
    ///
    /// `Zone.leds[offset()..offset() + num_leds()]` will return the LEDs in this segment.
    pub fn num_leds(&self) -> usize {
        self.data().led_count() as usize
    }

    /// Returns the index offset of this segment in the zone.
    ///
    /// `Zone.leds[offset()..offset() + num_leds()]` will return the LEDs in this segment.
    pub fn offset(&self) -> usize {
        self.data().offset() as usize
    }

    /// Returns a command to update the LEDs in this segment.
    pub fn update_leds_cmd(&self, colors: Vec<Color>) -> OpenRgbResult<UpdateCommand> {
        Ok(UpdateCommand::Segment {
            controller_id: self.zone.controller_id(),
            zone_id: self.zone.zone_id(),
            segment_id: self.segment_id,
            colors,
        })
    }
}

#[cfg(test)]
mod tests {
use crate::{OpenRgbClient, OpenRgbResult};

use super::*;

    #[tokio::test]
    async fn test_segment_set_leds() -> OpenRgbResult<()> {
        let client = OpenRgbClient::connect().await?;
        let controller = client.get_controller(5).await?;
        let zone = controller.get_zone(1)?;
        let seg = zone.get_segment(0)?;
        println!("seg.name(): {0:?}", seg.name());

        let colors = [
            Color::new(255, 255,0),
            Color::new(0, 255, 255),
            Color::new(255, 0, 255),
        ];

        let mut super_cmd = controller.cmd();
        for i in 0..3 {
            let seg = zone.get_segment(i)?;
            let cmd = seg.update_leds_cmd(vec![colors[i]; seg.num_leds()])?;
            super_cmd.add_command(cmd)?;
        }
        super_cmd.execute().await?;

        Ok(())
    }
}