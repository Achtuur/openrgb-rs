//! OpenRGB data types.
//!
//! See [OpenRGB SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.

mod color;
mod implement;
mod openrgb;
mod protocol_option;

pub use color::*;
pub use implement::*;
pub use openrgb::*;
pub use protocol_option::*;

/// Implements `TryFrom<u32>` and `From<$enum> for u32` for an enum with discriminants.
///
/// Previously this was derived using the Primitive crate, but that's a lot of overhead for such a simple feature
#[macro_export]
macro_rules! impl_enum_discriminant {
    ($enum: tt, $($var:ident: $value:expr),+) => {
        impl TryFrom<u32> for $enum {
            type Error = $crate::OpenRgbError;

            fn try_from(value: u32) -> core::result::Result<Self, Self::Error> {
                match value {
                    $(
                        $value => Ok($enum::$var),
                    )+
                    _ => Err($crate::OpenRgbError::ProtocolError(format!(
                        "unknown discriminant value {} for {}", value, stringify!($enum)
                    )))
                }
            }
        }

        impl From<$enum> for u32 {
            #[inline(always)]
            fn from(value: $enum) -> Self {
                u32::from(&value)
            }
        }

        impl<'a> From<&'a $enum> for u32 {
            #[inline(always)]
            fn from(value: &'a $enum) -> Self {
                match value {
                    $(
                        $enum::$var => $value,
                    )+
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro() {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        enum Test {
            A,
            B,
        }

        impl_enum_discriminant!(Test, A: 1, B: 2);

        assert_eq!(Test::try_from(1).unwrap(), Test::A);
        assert_eq!(Test::try_from(2).unwrap(), Test::B);
        assert!(Test::try_from(3).is_err());
        assert_eq!(u32::from(Test::A), 1);
        assert_eq!(u32::from(Test::B), 2);
        assert_eq!(u32::from(&Test::A), u32::from(Test::A));
        assert_eq!(u32::from(&Test::B), u32::from(Test::B));
    }
}