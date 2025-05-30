use crate::{
    OpenRgbError, OpenRgbResult,
    protocol::{ReadableStream, TryFromStream, Writable, WritableStream},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SegmentData {
    name: String,
    seg_type: i32,
    start_idx: u32,
    led_count: u32,
}

impl TryFromStream for SegmentData {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self> {
        if stream.protocol_version() < 4 {
            return Err(OpenRgbError::ProtocolError(
                "SegmentData is not supported in protocol version < 4".to_string(),
            ));
        }

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

impl Writable for SegmentData {
    fn size(&self) -> usize {
        self.name.size() + self.seg_type.size() + self.start_idx.size() + self.led_count.size()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        let mut n = 0;
        n += stream.write_value(&self.name).await?;
        n += stream.write_value(&self.seg_type).await?;
        n += stream.write_value(&self.start_idx).await?;
        n += stream.write_value(&self.led_count).await?;
        Ok(n)
    }
}
