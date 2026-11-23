#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::{cell::Cell, mem};

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[repr(C)]
struct Owned<'a> {
    tag: u8,
    tally: &'a Cell<u32>,
}

impl Drop for Owned<'_> {
    fn drop(&mut self) {
        self.tally
            .set(self.tally.get().wrapping_add(self.tag as u32));
    }
}

fn check() -> i32 {
    let (mut left, mut right) = (17_u32, 31_u32);
    mem::swap(&mut left, &mut right);
    if left != 31 || right != 17 {
        return 1;
    }

    let (mut empty_left, mut empty_right) = ((), ());
    mem::swap(&mut empty_left, &mut empty_right);

    let mut small = [5_u8; 67];
    let [first, second, .., last] = &mut small;
    *first = 9;
    *second = 4;
    *last = 41;
    let mut large = [8_u8; 67];
    mem::swap(&mut small, &mut large);
    if !matches!(small, [8, 8, .., 8]) || !matches!(large, [9, 4, .., 41]) {
        return 2;
    }

    let mut first = [17_u64; 17];
    let mut second = [31_u64; 17];
    mem::swap(&mut first, &mut second);
    if !matches!(first, [31, 31, .., 31]) || !matches!(second, [17, 17, .., 17]) {
        return 3;
    }

    let left_tally = Cell::new(0);
    let right_tally = Cell::new(0);
    {
        let mut left = Owned {
            tag: 5,
            tally: &left_tally,
        };
        let mut right = Owned {
            tag: 9,
            tally: &right_tally,
        };
        mem::swap(&mut left, &mut right);
        if left.tag != 9 || right.tag != 5 {
            return 4;
        }
        if left_tally.get() != 0 || right_tally.get() != 0 {
            return 5;
        }
    }
    if left_tally.get() != 5 || right_tally.get() != 9 {
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
