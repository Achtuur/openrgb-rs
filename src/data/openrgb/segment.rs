use crate::{data::TryFromStream, protocol::ReadableStream, OpenRgbResult};

#[derive(Debug, Eq, PartialEq)]
pub struct Segment {
    name: String,
    seg_type: i32,
    start_idx: u32,
    led_count: u32,
}

impl TryFromStream for Segment {
    async fn try_read(
        stream: &mut impl ReadableStream,
        protocol: u32,
    ) -> OpenRgbResult<Self> {
        let name = stream.read_value(protocol).await?;
        let seg_type = stream.read_value(protocol).await?;
        let start_idx = stream.read_value(protocol).await?;
        let led_count = stream.read_value(protocol).await?;

        Ok(Self {
            name,
            seg_type,
            start_idx,
            led_count,
        })
    }
}