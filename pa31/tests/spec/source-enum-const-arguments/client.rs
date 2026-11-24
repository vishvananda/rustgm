#![feature(no_core)]
#![no_core]

extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed_first() -> u8 {
    base::enum_first::<u8>()
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_second() -> u8 {
    base::enum_second::<u8>()
}
