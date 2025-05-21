use crate::{
    OpenRgbError, OpenRgbResult,
    protocol::{Writable, WritableStream},
};

impl<T: Writable, const N: usize> Writable for [T; N] {
    fn size(&self) -> usize {
        size_of::<u16>() + self.iter().map(|e| e.size()).sum::<usize>()
    }

    async fn try_write(self, stream: &mut impl WritableStream) -> OpenRgbResult<()> {
        stream
            .write_value(u16::try_from(self.len()).map_err(|e| {
                OpenRgbError::ProtocolError(format!("Array is too large to encode: {}", e))
            })?)
            .await?;
        for elem in self {
            stream.write_value(elem).await?;
        }
        Ok(())
    }
}
