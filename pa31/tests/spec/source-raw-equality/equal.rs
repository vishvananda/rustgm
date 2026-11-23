#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

fn same(left: &[u16; 4], right: &[u16; 4]) -> bool {
    left == right
}

fn check() -> i32 {
    let base = [0x103_u16, 0x80, 0xc800, 0xff];
    let equal = [0x103_u16, 0x80, 0xc800, 0xff];
    let first = [0x203_u16, 0x80, 0xc800, 0xff];
    let middle = [0x103_u16, 0x80, 0xc801, 0xff];
    let last = [0x103_u16, 0x80, 0xc800, 0x1ff];
    if !same(&base, &equal) || !same(&base, &base) {
        return 1;
    }
    if same(&base, &first) || same(&base, &middle) || same(&base, &last) {
        return 2;
    }

    let left: [u8; 0] = [];
    let right: [u8; 0] = [];
    if left != right || [true, false, true] == [true, false, false] {
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
