#![feature(no_core)]
#![no_core]

extern crate base;

const COMPILED: [u8; 3] = base::repeated(71);

#[unsafe(no_mangle)]
pub extern "C" fn entry(value: u8) -> i32 {
    let [a, b, c] = COMPILED;
    let [d, e, f] = base::repeated(value);
    match (a, b, c, d, e, f) {
        (71, 71, 71, 29, 29, 29) => 42,
        (71, 71, 71, 87, 87, 87) => 43,
        _ => 1,
    }
}
