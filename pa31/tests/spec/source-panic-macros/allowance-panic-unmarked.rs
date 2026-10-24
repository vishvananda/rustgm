#![feature(allow_internal_unstable)]
macro_rules! route { () => { core::panicking::panic("allowance proof") }; }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { if false { route!() }; 7u32 }
