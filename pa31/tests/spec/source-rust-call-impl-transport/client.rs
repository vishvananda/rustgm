#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed(value: u8) -> u8 {
    base::Identity(value)
}
