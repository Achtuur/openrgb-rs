use std::mem::MaybeUninit;

use crate::{
    protocol::{DeserFromBuf, ReceivedMessage}, OpenRgbResult, SerToBuf, WriteMessage
};

impl<T: SerToBuf, const N: usize> SerToBuf for [T; N] {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        self.as_slice().serialize(buf)
    }
}

impl <T: Copy + Default + DeserFromBuf + Sized, const N: usize> DeserFromBuf for [T; N] {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let mut arr = [const { MaybeUninit::<T>::uninit() }; N];

        for item in arr.iter_mut() {
            let d = T::deserialize(buf)?;
            item.write(d);
        }

        // the for loop either writes to every element of the array or returns an error
        unsafe {
            Ok(std::mem::transmute_copy(&arr))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DEFAULT_PROTOCOL;

    use super::*;

    #[test]
    fn test_deser_array() {
        let mut message = ReceivedMessage::new(&[0, 1, 2, 3, 4, 5], DEFAULT_PROTOCOL);
        let arr: [u8; 3] = message.read_value().unwrap();
        assert_eq!(arr, [0, 1, 2]);
        let arr2: [u8; 3] = message.read_value().unwrap();
        assert_eq!(arr2, [3, 4, 5]);
    }
}