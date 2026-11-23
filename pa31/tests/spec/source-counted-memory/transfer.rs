#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::intrinsics::{copy, copy_nonoverlapping, offset, write_bytes};

#[repr(C)]
struct Entry {
    first: u8,
    second: u8,
    third: u8,
}

fn fill() -> bool {
    let mut entries = [
        Entry {
            first: 1,
            second: 2,
            third: 3,
        },
        Entry {
            first: 4,
            second: 5,
            third: 6,
        },
        Entry {
            first: 7,
            second: 8,
            third: 9,
        },
        Entry {
            first: 10,
            second: 11,
            third: 12,
        },
    ];
    let base = (&raw mut entries) as *mut Entry;
    unsafe {
        write_bytes(offset(base, 1_usize), 0x55, 2);
        write_bytes(base, 0xfe, 0);
    }
    let [first, second, third, last] = entries;
    first.third == 3
        && second.first == 0x55
        && second.second == 0x55
        && second.third == 0x55
        && third.first == 0x55
        && third.second == 0x55
        && third.third == 0x55
        && last.first == 10
}

fn raw_byte() -> bool {
    let source = 0xfe_u8;
    let mut destination = 0_u8;
    // The bool pointers address raw storage that is copied as bytes.
    unsafe {
        copy_nonoverlapping(
            (&raw const source) as *const bool,
            (&raw mut destination) as *mut bool,
            1,
        )
    };
    destination == 0xfe
}

fn overlapping() -> bool {
    let mut storage = [2_u8, 3, 5, 7];
    let base = (&raw mut storage) as *mut u8;
    unsafe {
        let next = offset(base, 1_usize);
        copy(base, next, 3);
        let [a, b, c, d] = core::intrinsics::read_via_copy(base as *const [u8; 4]);
        if a != 2 || b != 2 || c != 3 || d != 5 {
            return false;
        }
        copy(next, base, 3);
    }
    let [a, b, c, d] = storage;
    a == 2 && b == 3 && c == 5 && d == 5
}

fn experiment() -> i32 {
    if !fill() || !raw_byte() || !overlapping() {
        return 1;
    }
    let mut zst = ();
    unsafe { write_bytes(&raw mut zst, 0xff, usize::MAX) };
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
