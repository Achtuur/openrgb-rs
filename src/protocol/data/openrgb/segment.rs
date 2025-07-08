use crate::{
    protocol::{stream2::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage}, ReadableStream, TryFromStream, Writable, WritableStream}, OpenRgbError, OpenRgbResult
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
        if stream.protocol_version() < 4 {
            return Err(OpenRgbError::ProtocolError(
                "SegmentData is not supported in protocol version < 4".to_string(),
            ));
        }
        let mut n = 0;
        n += stream.write_value(&self.name).await?;
        n += stream.write_value(&self.seg_type).await?;
        n += stream.write_value(&self.start_idx).await?;
        n += stream.write_value(&self.led_count).await?;
        Ok(n)
    }
}

impl DeserFromBuf for SegmentData {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        if buf.protocol_version() < 4 {
            return Err(OpenRgbError::ProtocolError(
                "SegmentData is not supported in protocol version < 4".to_string(),
            ));
        }

        let name = buf.read_value()?;
        let seg_type = buf.read_value()?;
        let start_idx = buf.read_value()?;
        let led_count = buf.read_value()?;

        Ok(Self {
            name,
            seg_type,
            start_idx,
            led_count,
        })
    }
}

impl SerToBuf for SegmentData {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        if buf.protocol_version() < 4 {
            return Err(OpenRgbError::ProtocolError(
                "SegmentData is not supported in protocol version < 4".to_string(),
            ));
        }
        buf.write_value(&self.name)?;
        buf.write_value(&self.seg_type)?;
        buf.write_value(&self.start_idx)?;
        buf.write_value(&self.led_count)?;
        Ok(())
    }
}