#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

#[inline(never)]
fn rotate_byte(word: u8, distance: u32) -> (u8, u8) {
    (word.rotate_left(distance), word.rotate_right(distance))
}

#[inline(never)]
fn rotate_signed(word: i64, distance: u32) -> (i64, i64) {
    (word.rotate_left(distance), word.rotate_right(distance))
}

#[inline(never)]
fn rotate_wide(word: u128, distance: u32) -> (u128, u128) {
    (word.rotate_left(distance), word.rotate_right(distance))
}

fn test() -> i32 {
    let (left, right) = rotate_byte(0x81, 0);
    if left != 0x81 || right != 0x81 {
        return 1;
    }
    let (left, right) = rotate_byte(0x81, 17);
    if left != 0x03 || right != 0xc0 {
        return 2;
    }
    let (left, right) = rotate_byte(0x81, u32::MAX);
    if left != 0xc0 || right != 0x03 {
        return 6;
    }
    let (left, right) = rotate_signed(i64::MIN, 65);
    if left != 1 || right != 0x4000000000000000 {
        return 3;
    }
    let (left, right) = rotate_wide(0x80000000000000000000000000000001, 255);
    if left != 0xc0000000000000000000000000000000 || right != 3 {
        return 4;
    }
    let (left, right) = rotate_wide(3, 128);
    if left != 3 || right != 3 {
        return 5;
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
    test()
}

#[cfg(not(guest_probe))]
fn main() {
    assert_eq!(test(), 0);
}
