#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[inline(never)]
fn add(left: u128, right: u128, carry: bool) -> (u128, bool) {
    left.carrying_add(right, carry)
}

#[inline(never)]
fn subtract(left: u128, right: u128, borrow: bool) -> (u128, bool) {
    left.borrowing_sub(right, borrow)
}

fn check() -> i32 {
    let max = u128::MAX;
    let (value, carry) = add(max, 3, false);
    if value != 2 || !carry {
        return 1;
    }
    let (value, carry) = add(max, 0, true);
    if value != 0 || !carry {
        return 2;
    }
    let (value, carry) = add(13, 24, true);
    if value != 38 || carry {
        return 3;
    }
    let (value, borrow) = subtract(0, 0, true);
    if value != max || !borrow {
        return 4;
    }
    let (value, borrow) = subtract(1, 3, false);
    if value != max - 1 || !borrow {
        return 5;
    }
    let (value, borrow) = subtract(12, 2, true);
    if value != 9 || borrow {
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
