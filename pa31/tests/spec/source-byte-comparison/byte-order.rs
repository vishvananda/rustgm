#![feature(core_intrinsics)]
#![allow(internal_features)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::intrinsics::compare_bytes;

#[inline(never)]
fn compare(left: &[u8; 4], right: &[u8; 4], count: usize) -> i32 {
    unsafe { compare_bytes(left.as_ptr(), right.as_ptr(), count) }
}

fn experiment() -> i32 {
    let left = [4, 128, 0, 255];
    let right = [4, 127, 255, 0];
    let tail = [4, 128, 0, 254];
    if compare(&left, &right, 0) == 0
        && compare(&left, &right, 1) == 0
        && compare(&left, &right, 2) > 0
        && compare(&right, &left, 4) < 0
        && compare(&left, &left, 4) == 0
        && compare(&left, &tail, 3) == 0
        && compare(&left, &tail, 4) > 0
    {
        0
    } else {
        1
    }
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    experiment()
}

#[cfg(not(guest_probe))]
fn main() {
    assert_eq!(experiment(), 0);
}
