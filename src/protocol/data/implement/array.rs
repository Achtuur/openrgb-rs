use crate::{
    OpenRgbResult,
    protocol::{Writable, WritableStream},
};

impl<T: Writable, const N: usize> Writable for [T; N] {
    fn size(&self) -> usize {
        self.as_slice().size()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<usize> {
        let mut n = 0;
        for item in self.iter() {
            n += item.try_write(stream).await?;
        }
        Ok(n)
    }
}

