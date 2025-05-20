use crate::{data::Writable, protocol::WritableStream, OpenRgbError, OpenRgbResult};

impl<T: Writable, const N: usize> Writable for [T; N] {
    fn size(&self, protocol: u32) -> usize {
        size_of::<u16>() + self.iter().map(|e| e.size(protocol)).sum::<usize>()
    }

    async fn try_write(
        self,
        stream: &mut impl WritableStream,
        protocol: u32,
    ) -> OpenRgbResult<()> {
        stream
            .write_value(
                u16::try_from(self.len())
                    .map_err(|e| OpenRgbError::ProtocolError(format!("Array is too large to encode: {}", e)))?,
                protocol,
            )
            .await?;
        for elem in self {
            stream.write_value(elem, protocol).await?;
        }
        Ok(())
    }
}