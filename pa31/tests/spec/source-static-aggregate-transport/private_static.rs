#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn read_private() -> u8 {
    let [first, _, _, _] = *base::DIGITS;
    first
}
