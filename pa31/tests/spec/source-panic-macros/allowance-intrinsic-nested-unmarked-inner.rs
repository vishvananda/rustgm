#![feature(allow_internal_unstable)]
macro_rules! inner { () => { core::intrinsics::likely(true) as u32 + 6u32 }; }
#[allow_internal_unstable(core_intrinsics)]
macro_rules! outer { () => { inner!() }; }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { outer!() }
