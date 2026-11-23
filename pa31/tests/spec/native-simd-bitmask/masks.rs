#![feature(core_intrinsics, repr_simd)]
#![allow(internal_features)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::intrinsics::simd::simd_bitmask;

#[repr(simd)]
struct I16x8([i16; 8]);
#[repr(simd)]
struct I8x32([i8; 32]);
#[repr(simd)]
struct I32x10([i32; 10]);
#[repr(simd)]
struct I64x4([i64; 4]);
#[repr(simd)]
struct I8x128([i8; 128]);

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

fn check() -> i32 {
    // Every input lane in this test is the documented zero or all-ones mask.
    let short: u8 = unsafe { simd_bitmask(I16x8([0, -1, -1, 0, -1, 0, 0, -1])) };
    if short != 0x96 {
        return 1;
    }

    let mut many = [0_i8; 32];
    many[0] = -1;
    many[8] = -1;
    many[30] = -1;
    many[31] = -1;
    let many: u32 = unsafe { simd_bitmask(I8x32(many)) };
    if many != 0xc000_0101 {
        return 2;
    }

    let uneven = [-1, -1, 0, -1, 0, 0, -1, 0, -1, 0];
    let word: u16 = unsafe { simd_bitmask(I32x10(uneven)) };
    let bytes: [u8; 2] = unsafe { simd_bitmask(I32x10(uneven)) };
    if word != 0x014b || !matches!(bytes, [0x4b, 0x01]) {
        return 3;
    }

    let wide_lanes: u8 = unsafe { simd_bitmask(I64x4([-1, 0, 0, -1])) };
    if wide_lanes != 9 {
        return 4;
    }

    let mut big = [0_i8; 128];
    big[64] = -1;
    big[127] = -1;
    let big: u128 = unsafe { simd_bitmask(I8x128(big)) };
    if big != ((1_u128 << 64) | (1_u128 << 127)) {
        return 5;
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
