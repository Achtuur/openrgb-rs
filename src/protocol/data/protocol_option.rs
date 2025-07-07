use crate::{TryFromStream, Writable};

/// Option that can be used to represent values not supported by the current protocol version.
///
/// If protocol version is suppported, this is just an `Option<T>`.
/// If not, then this is always `ProtocolOption::UnsupportedVersion`.
///
/// Useful when determining sizes of data structures that contains fields that may not be supported by the current protocol version.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum ProtocolOption<const VER: usize, T> {
    Some(T),
    UnsupportedVersion,
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


    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Some(v) => Some(v),
            Self::UnsupportedVersion => None,
        }
    }
}

impl<const VER: usize, T> TryFromStream for ProtocolOption<VER, T>
where T: TryFromStream
{
    async fn try_read(stream: &mut impl crate::ReadableStream) -> crate::OpenRgbResult<Self> {
        if stream.protocol_version() < VER as u32 {
            return Ok(ProtocolOption::UnsupportedVersion);
        }
        let val = stream.read_value::<T>().await?;
        Ok(ProtocolOption::Some(val))
    }
}

impl<const VER: usize, T> Writable for ProtocolOption<VER, T>
where T: Writable
{
    fn size(&self) -> usize {
        match self {
            Self::Some(v) => v.size(),
            Self::UnsupportedVersion => 0, // No size if unsupported
        }
    }

    async fn try_write(&self, stream: &mut impl crate::WritableStream) -> crate::OpenRgbResult<usize> {
        match self {
            Self::Some(v) => v.try_write(stream).await,
            Self::UnsupportedVersion => Ok(0), // No write if unsupported
        }
    }
}