#![feature(no_core)]
#![no_core]

extern crate base;

const IMPORTED: u8 = base::value::<13>();

#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match (IMPORTED, base::value::<17>(), base::value::<29>()) {
        (13, 17, 29) => 42,
        _ => 1,
    }
}
