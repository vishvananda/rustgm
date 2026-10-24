#![feature(allow_internal_unstable)]
#[allow_internal_unstable(core_intrinsics)]
macro_rules! route { ($f:path) => { $f(true) as u32 + 6u32 }; }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { route!(core::intrinsics::likely) }
