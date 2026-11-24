#![deny(deprecated_in_future)]

pub use crate as same;

pub fn as_identity(value: same::Number) -> u8 {
    value
}

pub type Number = u8;
