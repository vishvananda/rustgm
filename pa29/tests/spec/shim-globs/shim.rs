// The rustc_std_workspace_core shape: a no_core crate that re-exports its
// `core` dependency by one glob. Its clients name it `core`.
#![feature(no_core)]
#![no_core]
pub use core::*;
