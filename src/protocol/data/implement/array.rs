use crate::{
    protocol::{stream2::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage}, Writable, WritableStream}, OpenRgbResult
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

impl<T: SerToBuf, const N: usize> SerToBuf for [T; N] {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        self.as_slice().serialize(buf)
    }
}

impl <T: Copy + Default + DeserFromBuf, const N: usize> DeserFromBuf for [T; N] {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let mut arr = [T::default(); N];
        for item in arr.iter_mut() {
            *item = T::deserialize(buf)?;
        }
        Ok(arr)
    }
}