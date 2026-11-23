#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

const INVALID: u32 = unsafe { core::intrinsics::exact_div(7_u32, 3_u32) };

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    INVALID as i32
}

#[cfg(not(guest_probe))]
fn main() {
    assert_eq!(INVALID, 2);
}
