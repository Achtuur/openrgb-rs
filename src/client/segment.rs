use crate::{client::command::UpdateCommand, data::SegmentData, Color, OpenRgbResult, Zone};

pub struct Segment<'z> {
    zone: &'z Zone<'z>,
    segment_id: usize,
}

impl<'z> Segment<'z> {
    pub(crate) fn new(zone: &'z Zone<'z>, segment_id: usize) -> Self {
        Self { zone, segment_id }
    }

    pub fn id(&self) -> usize {
        self.segment_id
    }

    pub fn name(&self) -> &str {
        self.data().name()
    }

    pub fn data(&self) -> &SegmentData {
        self.zone.data()
        .segments
        .value()
        .expect("Segment struct created with protocol version < 4")
        .get(self.segment_id)
        .expect("Segment data not found")
    }

    pub fn num_leds(&self) -> usize {
        self.data().led_count() as usize
    }

    pub fn offset(&self) -> usize {
        self.data().offset() as usize
    }

    pub fn update_leds_cmd(&self, colors: Vec<Color>) -> OpenRgbResult<UpdateCommand> {
        Ok(UpdateCommand::Segment {
            controller_id: self.zone.controller_id(),
            zone_id: self.zone.zone_id(),
            segment_id: self.segment_id,
            colors,
        })

        // let mut cmd = UpdateLedCommand::new(self.controller);
        // cmd.add_update_zone(self.zone_id, colors)?;
        // Ok(cmd)
    }
}

#[cfg(test)]
mod tests {
use crate::{OpenRgbClientWrapper, OpenRgbResult};

use super::*;

    #[tokio::test]
    async fn test_segment_set_leds() -> OpenRgbResult<()> {
        let mut client = OpenRgbClientWrapper::connect().await?;
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