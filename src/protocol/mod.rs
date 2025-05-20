mod packet;
mod stream;

pub use {
    packet::*,
    stream::*,
};

pub(crate) const MAGIC: [u8; 4] = *b"ORGB";