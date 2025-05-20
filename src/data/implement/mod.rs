//! Implements the readable and writable traits for core/std types.

mod primitive;
mod string;
mod tuple;
mod vec;

pub use {
    primitive::*,
    string::*,
    tuple::*,
    vec::*,
};