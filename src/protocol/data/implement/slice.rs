use crate::{protocol::Writable, WritableStream};

impl<T: Writable> Writable for &[T] {
    fn size(&self) -> usize {
        size_of::<u16>() + self.iter().map(|item| item.size()).sum::<usize>()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> crate::OpenRgbResult<usize> {
        let len = u16::try_from(self.len()).map_err(|e| {
            crate::OpenRgbError::ProtocolError(format!("Slice is too large to encode: {}", e))
        })?;
        let mut n = stream.write_value(&len).await?;
        for elem in self.iter() {
            n += stream.write_value(elem).await?;
        }
        Ok(n)
    }
}
