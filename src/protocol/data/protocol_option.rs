use crate::{protocol::serialize::{DeserFromBuf, ReceivedMessage}, SerToBuf, WriteMessage};

/// Option that can be used to represent values not supported by the current protocol version.
///
/// If protocol version is suppported, this is just an `T`.
/// If not, then this is always `ProtocolOption::UnsupportedVersion`.
///
/// Useful when determining sizes of data structures that contains fields that may not be supported by the current protocol version.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProtocolOption<const VER: usize, T> {
    Some(T),
    UnsupportedVersion,
}

impl<const VER: usize, T: Default> std::default::Default for ProtocolOption<VER, T> {
    fn default() -> Self {
        ProtocolOption::Some(T::default())
    }
}

impl<const VER: usize, T> From<ProtocolOption<VER, T>> for Option<T> {
    fn from(value: ProtocolOption<VER, T>) -> Self {
        match value {
            ProtocolOption::Some(v) => Some(v),
            ProtocolOption::UnsupportedVersion => None,
        }
    }
}

impl<const VER: usize, T> ProtocolOption<VER, T> {
    pub fn new(val: T, version: usize) -> Self {
        if version < VER {
            return Self::UnsupportedVersion;
        }
        Self::Some(val)
    }

    pub fn some(val: T) -> Self {
        Self::Some(val)
    }

    pub fn replace(&mut self, val: T) {
        match self {
            Self::Some(_) => *self = Self::Some(val),
            Self::UnsupportedVersion => (),
        }
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Some(v) => Some(v),
            Self::UnsupportedVersion => None,
        }
    }
}

impl<const VER: usize, T> DeserFromBuf for ProtocolOption<VER, T>
where T: DeserFromBuf
{
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        if buf.protocol_version() < VER as u32 {
            return Ok(ProtocolOption::UnsupportedVersion);
        }
        let val = T::deserialize(buf)?;
        Ok(ProtocolOption::Some(val))
    }
}

impl<const VER: usize, T> SerToBuf for ProtocolOption<VER, T>
where T: SerToBuf
{
    fn serialize(&self, buf: &mut WriteMessage) -> crate::OpenRgbResult<()> {
        if buf.protocol_version() < VER as u32 {
            return Ok(())
        }

        match self {
            Self::Some(v) => v.serialize(buf),
            Self::UnsupportedVersion => Ok(()), // No write if this came from unsupported read
        }
    }
}