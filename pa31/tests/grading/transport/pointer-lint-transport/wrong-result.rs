#![feature(no_core)]
#![no_core]

extern crate base;

pub fn wrong(value: *const u8) -> u8 {
    base::pointer(value)
}
