
use crate::OpenRgbResult;


mod client;
pub(crate) mod data;
mod packet;
mod stream;
#[cfg(test)]
mod tests;

pub(crate) use {
    client::*,
    packet::*,
    stream::*,
};

pub use data::Color;

/// Things that can read from stream to construct itself
/// TryFromStream is actually what it is
#[doc(hidden)]
pub trait TryFromStream: Sized + Send + Sync {
    async fn try_read(stream: &mut impl ReadableStream) -> OpenRgbResult<Self>;
}

/// Things that can write itself to a stream
#[doc(hidden)]
pub trait Writable: Sized + Send + Sync {
    fn size(&self) -> usize;
    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<()>;
}

impl<T: Writable> Writable for &T {
    fn size(&self) -> usize {
        (*self).size()
    }

    async fn try_write(&self, stream: &mut impl WritableStream) -> OpenRgbResult<()> {
        (*self).try_write(stream).await
    }
}