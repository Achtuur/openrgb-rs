use crate::{protocol::{ReadableStream, TryFromStream, Writable, WritableStream}, OpenRgbResult};

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
    ) -> OpenRgbResult<Self> {
        let name = stream.read_value().await?;
        let seg_type = stream.read_value().await?;
        let start_idx = stream.read_value().await?;
        let led_count = stream.read_value().await?;

        Ok(Self {
            name,
            seg_type,
            start_idx,
            led_count,
        })
    }
}

impl Writable for Segment {
    fn size(&self) -> usize {
        self.name.size() + self.seg_type.size() + self.start_idx.size() + self.led_count.size()
    }

    async fn try_write(
        self,
        stream: &mut impl WritableStream,
    ) -> OpenRgbResult<()> {
        stream.write_value(self.name).await?;
        stream.write_value(self.seg_type).await?;
        stream.write_value(self.start_idx).await?;
        stream.write_value(self.led_count).await?;

        Ok(())
    }
}