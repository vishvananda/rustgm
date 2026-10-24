#![feature(allow_internal_unstable)]
#[allow_internal_unstable(core_intrinsics)]
macro_rules! route { () => { 7u32 }; }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { let _ = route!(); core::intrinsics::likely(true) as u32 + 6u32 }
