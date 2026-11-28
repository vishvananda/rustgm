#![feature(no_core)]
#![no_core]

extern crate base;

pub fn read(value: &mut base::Shared) -> u8 {
    base::require(&value)
}
