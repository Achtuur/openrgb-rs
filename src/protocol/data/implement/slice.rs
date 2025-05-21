use crate::protocol::Writable;

impl<'a, T: Writable> Writable for &'a [T]
{
    fn size(&self) -> usize {
        size_of::<u16>() + self.iter().map(|item| item.size()).sum::<usize>()
    }

    async fn try_write(&self, stream: &mut impl crate::protocol::WritableStream) -> crate::OpenRgbResult<()> {
        let len = u16::try_from(self.len()).map_err(|e| {
            crate::OpenRgbError::ProtocolError(format!("Slice is too large to encode: {}", e))
        })?;
        stream.write_value(&len).await?;
        for elem in self.iter() {
            stream.write_value(elem).await?;
        }
        Ok(())
    }
}