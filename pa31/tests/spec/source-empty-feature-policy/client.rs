#![feature(no_core)]
#![no_core]

#[unsafe(no_mangle)]
pub extern "C" fn observe(input: u8) -> u8 {
    base::stable_byte(input)
}
