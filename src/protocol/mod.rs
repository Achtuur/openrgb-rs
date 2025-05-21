mod packet;
mod stream;
mod client;

use {crate::{OpenRgbError, OpenRgbResult}};
pub use {
    packet::*,
    stream::*,
    client::*,
};


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
    async fn try_write(
        self,
        stream: &mut impl WritableStream,
    ) -> OpenRgbResult<()>;
}