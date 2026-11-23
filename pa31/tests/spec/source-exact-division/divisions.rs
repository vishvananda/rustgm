#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::intrinsics::exact_div;

const QUOTIENT: i16 = unsafe { exact_div(-147_i16, 7_i16) };

#[unsafe(no_mangle)]
pub unsafe fn signed(left: i8, right: i8) -> i8 {
    unsafe { exact_div(left, right) }
}

#[unsafe(no_mangle)]
pub unsafe fn unsigned(left: usize, right: usize) -> usize {
    unsafe { exact_div(left, right) }
}

fn experiment() -> i32 {
    if QUOTIENT != -21 {
        return 1;
    }
    if unsafe { signed(-84, 7) } != -12 || unsafe { signed(84, -7) } != -12 {
        return 2;
    }
    if unsafe { unsigned(200, 25) } != 8 {
        return 3;
    }
    let wide = 0x8000_0000_0000_0000_0000_0000_0000_0000_u128;
    if unsafe { exact_div(wide, 16_u128) } != 0x0800_0000_0000_0000_0000_0000_0000_0000_u128 {
        return 4;
    }
    0
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
