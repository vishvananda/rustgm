#![no_std]
mod legacy { use core::option::Option; pub fn none() -> Option<u8> { None } }
pub use legacy::none;
