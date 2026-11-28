#![deny(unreachable_pub)]
mod inner { pub fn visible() -> u8 { 7 } }
pub use inner::visible;
