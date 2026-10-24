#![feature(allow_internal_unstable)]
#[allow_internal_unstable(core_intrinsics)]
macro_rules! route { ($($t:tt)*) => { $($t)* }; }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { route!(core::intrinsics::likely(true) as u32 + 6u32) }
