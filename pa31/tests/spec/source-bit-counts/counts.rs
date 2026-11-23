#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

const ONES: u32 = core::intrinsics::ctpop(0b1011_u8);

fn experiment() -> i32 {
    if ONES != 3 || (0xf1_u8).count_ones() != 5 || (0_u8).leading_zeros() != 8 {
        return 1;
    }
    let word = 0x0400_u16;
    if word.leading_zeros() != 5 || word.trailing_zeros() != 10 {
        return 2;
    }
    if (-1_i64).count_ones() != 64 || (0_u32).trailing_zeros() != 32 {
        return 3;
    }
    let wide = 0x8000_0000_0000_0000_0000_0000_0000_0020_u128;
    if wide.count_ones() != 2 || wide.leading_zeros() != 0 || wide.trailing_zeros() != 5 {
        return 4;
    }
    if (0x1234_u16).swap_bytes() != 0x3412 || (0x09_u8).reverse_bits() != 0x90 {
        return 6;
    }
    let (leading, trailing) = unsafe {
        (
            core::intrinsics::ctlz_nonzero(0x80_u8),
            core::intrinsics::cttz_nonzero(word),
        )
    };
    if leading == 0 && trailing == 10 { 0 } else { 5 }
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
