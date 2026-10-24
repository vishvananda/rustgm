#![feature(allow_internal_unstable)]
#[allow_internal_unstable(panic_internals, core_intrinsics)]
macro_rules! route { () => {{ if false { core::panicking::panic("allowance proof") }; core::intrinsics::likely(true) as u32 + 6u32 }}; }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { route!() }
