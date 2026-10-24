#![feature(allow_internal_unstable)]
#![feature(core_intrinsics)]
#[allow_internal_unstable(core_intrinsics)]
macro_rules! route { ($e:expr) => { $e }; }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { route!(core::intrinsics::likely(true) as u32 + 6u32) }
