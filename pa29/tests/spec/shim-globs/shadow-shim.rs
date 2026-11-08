// The shim's own `option` module shadows the one its glob would bind.
#![feature(no_core)]
#![no_core]
pub use core::*;
pub mod option {}
