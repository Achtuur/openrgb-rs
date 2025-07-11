use crate::{
    Color, Controller, OpenRgbError, OpenRgbResult,
    client::{command::UpdateCommand, segment::Segment},
    data::{SegmentData, ZoneData},
};

/// A zone in a controller, which contains one or more LEDs.
///
/// Zones can also contain segments, which are user-created subdivisions of the zone.
pub struct Zone<'a> {
    zone_id: usize,
    controller: &'a Controller,
}

impl<'a> Zone<'a> {
    pub(crate) fn new(controller: &'a Controller, zone_id: usize) -> Self {
        Self {
            zone_id,
            controller,
        }
    }

    /// Returns the ID of the controller this zone belongs to.
    pub fn controller_id(&self) -> usize {
        self.controller.id()
    }

    /// Returns the ID of this zone.
    pub fn zone_id(&self) -> usize {
        self.zone_id
    }

    /// Returns the `ZoneData` for this zone
    pub fn data(&self) -> &ZoneData {
        // `Zone` can only be created if the zone is valid, so this zone must always exist
        self.controller
            .data()
            .zones
            .get(self.zone_id)
            .expect("Invalid zone was created") // should be unreachable
    }

    /// Returns the segment with the given `segment_id`.
    pub fn get_segment(&'a self, segment_id: usize) -> OpenRgbResult<Segment<'a>> {
        let is_valid = self
            .data()
            .segments
            .value()
            .is_some_and(|seg| segment_id < seg.len());
        if !is_valid {
            return Err(OpenRgbError::CommandError(format!(
                "Segment with id {segment_id} not found in zone {}",
                self.zone_id
            )));
        }
        Ok(Segment::new(self, segment_id))
    }

    /// Returns an iterator over all segments in this zone.
    pub fn get_all_segments(&'a self) -> impl Iterator<Item = Segment<'a>> {
        self.data()
            .segments
            .value()
            .into_iter()
            .flatten()
            .enumerate()
            .map(move |(id, _)| Segment::new(self, id))
    }

    /// Returns the number of leds in this zone.
    pub fn num_leds(&self) -> usize {
        self.data().leds_count as usize
    }

    /// Returns the offset of this zone in the controller's LED array.
    pub fn offset(&self) -> usize {
        self.controller
            .get_zone_led_offset(self.zone_id)
            .expect("Zone id should be valid")
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
    }

    /// Sets a single LED in this zone to the given `color`.
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds for this zone.
    pub async fn set_led(&self, idx: usize, color: Color) -> OpenRgbResult<()> {
        if idx >= self.num_leds() {
            return Err(OpenRgbError::CommandError(format!(
                "Index {idx} out of bounds for zone {} with {} LEDs",
                self.zone_id,
                self.num_leds()
            )));
        }
        let idx = self.offset() + idx;
        self.controller.set_led(idx, color).await
    }

    /// Sets all LEDs in this zone to the given `color`.
    pub async fn set_all_leds(&self, color: Color) -> OpenRgbResult<()> {
        let colors = vec![color; self.data().leds_count as usize];
        self.set_leds(colors).await
    }

    /// Sets the LEDs in this zone to the given colors.
    pub async fn set_leds(&self, colors: impl IntoIterator<Item = Color>) -> OpenRgbResult<()> {
        let color_v = colors.into_iter().collect::<Vec<_>>();
        if color_v.len() >= self.num_leds() {
            tracing::warn!(
                "Zone {} for controller {} was given {} colors, while its length is {}. This might become a hard error in the future.",
                self.zone_id,
                self.controller.name(),
                color_v.len(),
                self.num_leds()
            );
        }
        self.controller.set_zone_leds(self.zone_id, color_v).await
    }

    /// Adds a segment to this zone.
    pub async fn add_segment(
        &self,
        name: impl Into<String>,
        start_idx: usize,
        led_count: usize,
    ) -> OpenRgbResult<()> {
        if start_idx + led_count > self.num_leds() {
            return Err(OpenRgbError::CommandError(format!(
                "Segment start index {} + count {} exceeds zone LED count {}",
                start_idx,
                led_count,
                self.num_leds()
            )));
        }

        let data = SegmentData::new(name.into(), start_idx as u32, led_count as u32);
        self.controller
            .proto()
            .add_segment(self.controller.id() as u32, self.zone_id as u32, &data)
            .await
    }

    /// Clears the segments for this CONTROLLER.
    /// This clears all segments for all zones of the controller, not just this zone.
    pub async fn clear_segments(&self) -> OpenRgbResult<()> {
        self.controller.clear_segments().await
    }

    /// Resizes this zone to a new size.
    pub async fn resize(&self, new_size: usize) -> OpenRgbResult<()> {
        self.controller
            .proto()
            .resize_zone(
                self.controller.id() as u32,
                self.zone_id as u32,
                new_size as u32,
            )
            .await
    }
}
