#![feature(no_core)]
#![no_core]

extern crate base;

const IMPORTED: u8 = base::value::<13>();
const BYTES: [u8; 2] = base::bytes::<2>();

#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let value = 51_u8;
    let borrowed = *base::reborrow(&value);
    let dispatch = base::invoke(61_u8);
    let [left, right] = BYTES;
    match (IMPORTED, base::value::<17>(), base::value::<29>(), borrowed, left, right, dispatch) {
        (13, 17, 29, 51, 7, 7, 61) => 42,
        _ => 1,
    }
}
