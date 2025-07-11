use crate::{
    client::{command::UpdateCommand, segment::Segment}, data::ZoneData, Color, Controller, OpenRgbError, OpenRgbResult
};

pub struct Zone<'a> {
    zone_id: usize,
    controller: &'a Controller,
}

impl<'a> Zone<'a> {
    pub(crate) fn new(
        controller: &'a Controller,
        zone_id: usize,
    ) -> Self {
        Self {
            zone_id,
            controller,
        }
    }

    pub fn controller_id(&self) -> usize {
        self.controller.id()
    }

    pub fn zone_id(&self) -> usize {
        self.zone_id
    }

    pub fn data(&self) -> &ZoneData {
        // `Zone` can only be created if the zone is valid, so this zone must always exist
        self.controller
        .data()
        .zones
        .get(self.zone_id)
        .expect("Invalid zone was created") // should be unreachable
    }

    pub fn get_segment(&'a self, segment_id: usize) -> OpenRgbResult<Segment<'a>> {
        let is_valid = self.data().segments.value().is_some_and(|seg| segment_id < seg.len());
        if !is_valid {
            return Err(OpenRgbError::CommandError(format!(
                "Segment with id {segment_id} not found in zone {}",
                self.zone_id
            )));
        }
        Ok(Segment::new(self, segment_id))
    }

    pub fn num_leds(&self) -> usize {
        self.data().leds_count as usize
    }

    /// Returns the offset of this zone in the controller's LED array.
    pub fn offset(&self) -> usize {
        self.controller.get_zone_led_offset(self.zone_id).expect("Zone id should be valid")
    }

    /// Resizes `colors` to fit the number of LEDs in this zone.
    pub fn resize_colors_to_fit(&self, colors: &mut Vec<Color>) {
        colors.resize(self.num_leds(), Color::default());
    }

    /// Returns an iterator that yields a color for each LED in the zone.
    ///
    /// If `color` is too short, it will be padded with black.
    /// If `colors` is longer than the number of leds in the zone, it is truncated.
    pub fn zone_colors_from_iter<I>(&self, colors: I) -> impl Iterator<Item = Color> + use<I>
    where
        I: IntoIterator<Item = Color>,
    {
        let mut colors = colors.into_iter();
        (0..self.num_leds()).map(move |_| colors.next().unwrap_or_default())

    }

    /// Returns a command to update the LEDs for this Zone to `colors`.
    ///
    /// The command must be executed by calling `.execute()`
    pub fn update_leds_cmd(&'a self, colors: Vec<Color>) -> OpenRgbResult<UpdateCommand> {
        Ok(UpdateCommand::Zone {
            controller_id: self.controller.id(),
            zone_id: self.zone_id,
            colors,
        })

        // let mut cmd = UpdateLedCommand::new(self.controller);
        // cmd.add_update_zone(self.zone_id, colors)?;
        // Ok(cmd)
    }

    pub async fn update_leds(&self, colors: &[Color]) -> OpenRgbResult<()> {
        // if colors.len() != self.data.leds_count as usize {
        //     return Err(OpenRgbError::ProtocolError(format!(
        //         "Invalid number of colors: expected {}, got {}",
        //         self.data.leds_count,
        //         colors.len()
        //     )));
        // }
        let colors = self.zone_colors_from_iter(colors.iter().copied())
        .collect::<Vec<_>>();

        self.controller.update_zone(self.zone_id, &colors).await
    }

    pub async fn update_leds_uniform(&self, color: Color) -> OpenRgbResult<()> {
        let colors = vec![color; self.data().leds_count as usize];
        self.update_leds(&colors).await
    }
}
