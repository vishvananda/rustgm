#![feature(allow_internal_unstable)]
#[allow_internal_unstable(panic_internals)]
macro_rules! route { ($e:expr) => { $e }; }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { if false { route!(core::panicking::panic("allowance proof")) }; 7u32 }
