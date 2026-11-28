#![deny(unreachable_pub)]
mod inner { pub fn hidden() -> u8 { 7 } }
pub fn entry() -> u8 { inner::hidden() }
