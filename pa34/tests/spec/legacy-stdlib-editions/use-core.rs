#![no_std]
mod legacy { use core::fmt; pub fn done() -> fmt::Result { Ok(()) } }
pub use legacy::done;
