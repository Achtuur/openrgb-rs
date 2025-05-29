use crate::{
    OpenRgbResult,
    protocol::{Writable, WritableStream},
};

impl<T: Writable, const N: usize> Writable for [T; N] {
    fn size(&self) -> usize {
        self.as_slice().size()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<()> {
        self.as_slice().try_write(stream).await
    }
}
