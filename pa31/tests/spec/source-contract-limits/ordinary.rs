#![feature(allow_internal_unstable)]
#[allow_internal_unstable()]
macro_rules! route { () => { 7u8 }; }
pub fn value() -> u8 { route!() }
