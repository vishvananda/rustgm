// Two globs give `shared` different targets; it is ambiguous only where used.
#![feature(no_core)]
#![no_core]
pub use core::*;
pub use core::left::*;
pub use core::right::*;
