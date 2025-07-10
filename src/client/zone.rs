use crate::{
    Color, OpenRgbError, OpenRgbProtocol, OpenRgbResult, data::ZoneData,
};

pub struct Zone<'a> {
    zone_id: u32,
    controller_id: u32,
    proto: &'a OpenRgbProtocol,
    data: &'a ZoneData,
}

impl<'a> Zone<'a> {
    pub fn new(
        controller_id: u32,
        zone_id: u32,
        proto: &'a OpenRgbProtocol,
        data: &'a ZoneData,
    ) -> Self {
        Self {
            zone_id,
            controller_id,
            proto,
            data,
        }
    }

    pub fn controller_id(&self) -> u32 {
        self.controller_id
    }

    pub fn zone_id(&self) -> u32 {
        self.zone_id
    }

    pub fn num_leds(&self) -> usize {
        self.data.leds_count as usize
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

        self.proto
            .update_zone_leds(self.controller_id(), self.zone_id(), &colors)
            .await
    }

    pub async fn update_leds_uniform(&self, color: Color) -> OpenRgbResult<()> {
        let colors = vec![color; self.data.leds_count as usize];
        self.update_leds(&colors).await
    }
}
