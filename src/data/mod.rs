//! OpenRGB data types.
//!
//! See [OpenRGB SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.

pub use color::*;
pub use implement::*;
pub use openrgb::*;

use crate::protocol::{ReadableStream, WritableStream};
use crate::{OpenRgbError, OpenRgbResult};

mod color;
mod implement;
mod openrgb;

/// Things that can read from stream to construct itself
/// TryFromStream is actually what it is
#[doc(hidden)]
pub trait TryFromStream: Sized + Send + Sync {
    async fn try_read(
        stream: &mut impl ReadableStream,
        protocol: u32,
    ) -> Result<Self, OpenRgbError>;
}

/// Things that can write itself to a stream
#[doc(hidden)]
pub trait Writable: Sized + Send + Sync {
    fn size(&self, protocol: u32) -> usize;
    async fn try_write(
        self,
        stream: &mut impl WritableStream,
        protocol: u32,
    ) -> OpenRgbResult<()>;
}