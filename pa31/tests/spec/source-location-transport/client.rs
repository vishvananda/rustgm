#![feature(no_core)]
#![no_core]

extern crate base;

fn main() {}

#[unsafe(no_mangle)]
pub extern "C" fn add(left: u8, right: u8) -> u8 {
    left + right
}

#[unsafe(no_mangle)]
pub extern "C" fn call_tracked(value: u8) -> u8 {
    base::forwarded(value)
}
