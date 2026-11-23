#![feature(core_intrinsics, repr_simd)]
#![allow(internal_features)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

use core::intrinsics::simd::{
    simd_and, simd_bitmask, simd_eq, simd_or, simd_reduce_all, simd_splat,
};

#[derive(Clone, Copy)]
#[repr(simd)]
struct I16x8([i16; 8]);

#[derive(Clone, Copy)]
#[repr(simd)]
struct I64x4([i64; 4]);

#[derive(Clone, Copy)]
#[repr(simd)]
struct I32x10([i32; 10]);

fn check() -> i32 {
    let values = I16x8([10, 10, 10, 0, 10, 0, 0, 10]);
    let tens: I16x8 = unsafe { simd_eq(values, simd_splat(10_i16)) };
    let zeroes: I16x8 = unsafe { simd_eq(values, simd_splat(0_i16)) };
    if unsafe { simd_bitmask::<_, u8>(tens) } != 0x97
        || unsafe { simd_reduce_all(tens) }
        || !unsafe { simd_reduce_all(simd_or(tens, zeroes)) }
    {
        return 1;
    }

    let values = I64x4([i64::MIN, 8, i64::MIN, i64::MAX]);
    let minimum: I64x4 = unsafe { simd_splat(i64::MIN) };
    let selected: I64x4 = unsafe { simd_eq(values, minimum) };
    let all: I64x4 = unsafe { simd_eq(minimum, minimum) };
    if unsafe { simd_bitmask::<_, u8>(selected) } != 5
        || !unsafe { simd_reduce_all(all) }
        || !unsafe { simd_reduce_all(simd_or(selected, I64x4([0, -1, 0, -1]))) }
    {
        return 2;
    }

    let a = I32x10([1, 1, 0, 0, 0, 0, 0, 0, 0, 1]);
    let b = I32x10([1, 0, 0, 0, 0, 0, 0, 0, 1, 1]);
    let ones: I32x10 = unsafe { simd_splat(1_i32) };
    let a: I32x10 = unsafe { simd_eq(a, ones) };
    let b: I32x10 = unsafe { simd_eq(b, ones) };
    if unsafe { simd_bitmask::<_, u16>(simd_and(a, b)) } != 0x0201 {
        return 3;
    }

    0
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
