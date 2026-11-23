#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[inline(never)]
fn full_limb_product(a: u128, b: u128, addend: u128, carry: u128) -> (u128, u128) {
    a.carrying_mul_add(b, addend, carry)
}

fn check() -> i32 {
    let max = u128::MAX;
    let (low, high) = full_limb_product(max, max, max, max);
    if low != max || high != max {
        return 1;
    }
    let (low, high) = full_limb_product(max, 2, 3, 5);
    if low != 6 || high != 2 {
        return 2;
    }
    let limb = 1_u128 << 64;
    let (low, high) = full_limb_product(limb + 1, limb + 2, limb - 1, 5);
    if low != 4 * limb + 6 || high != 1 {
        return 3;
    }
    let (low, high) = 255_u8.carrying_mul_add(254, 253, 252);
    if low != 251 || high != 254 {
        return 4;
    }
    let (low, high) = u64::MAX.carrying_mul_add(2, 0, 1);
    if low != u64::MAX || high != 1 {
        return 5;
    }
    let (low, high) = usize::MAX.carrying_mul_add(0, usize::MAX, 1);
    if low != 0 || high != 1 {
        return 6;
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
