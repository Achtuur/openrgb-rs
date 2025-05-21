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
