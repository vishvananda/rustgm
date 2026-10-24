#![feature(allow_internal_unstable)]
macro_rules! inner { ($e:expr) => { $e }; }
#[allow_internal_unstable(core_intrinsics)]
macro_rules! outer { () => { inner!(core::intrinsics::likely(true) as u32 + 6u32) }; }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { outer!() }
