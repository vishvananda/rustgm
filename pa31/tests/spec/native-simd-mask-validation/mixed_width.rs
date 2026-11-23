#![feature(core_intrinsics, repr_simd)]
#![allow(internal_features)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

use core::intrinsics::simd::{simd_bitmask, simd_eq};

#[repr(simd)]
struct I16x8([i16; 8]);

#[repr(simd)]
struct I8x8([i8; 8]);

#[inline(never)]
fn compared(a: I16x8, b: I16x8) -> I8x8 {
    unsafe { simd_eq(a, b) }
}

fn check() -> i32 {
    let masks = compared(I16x8([2, 3, 4, 5, 8, 13, 21, 34]), I16x8([2, 0, 4, 0, 0, 13, 0, 34]));
    let bits: u8 = unsafe { simd_bitmask(masks) };
    if bits == 0xa5 { 0 } else { 1 }
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    check()
}

#[cfg(not(guest_probe))]
fn main() {
    std::process::exit(check());
}
