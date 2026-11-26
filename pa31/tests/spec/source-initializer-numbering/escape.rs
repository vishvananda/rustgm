#![feature(no_core)]
#![no_core]
extern crate base;

pub fn escape<'a>(_anchor: &'a u32) -> &'a u32 {
    let value = 7;
    (base::BORROW)(&value)
}
