use crate::protocol::{SerToBuf, WriteMessage};

impl<T: SerToBuf> SerToBuf for &[T] {
    fn serialize(&self, buf: &mut WriteMessage) -> crate::OpenRgbResult<()> {
        let len = u16::try_from(self.len()).map_err(|e| {
            crate::OpenRgbError::ProtocolError(format!("Slice is too large to encode: {e}"))
        })?;
        buf.write_u16(len);
        for item in self.iter() {
            item.serialize(buf)?;
        }
        Ok(())
    }
}