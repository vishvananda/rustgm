#![feature(no_core)]
#![no_core]

extern crate base;

fn main() {
    let _ = <u8 as base::Holder<bool>>::carry::<u8>(7u8);
}
