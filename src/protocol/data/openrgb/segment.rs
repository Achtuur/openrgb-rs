use crate::{
    protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage}, OpenRgbError, OpenRgbResult
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SegmentData {
    name: String,
    seg_type: i32,
    start_idx: u32,
    led_count: u32,
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