#![feature(no_core)]
#![no_core]

extern crate base;

fn main() {
    let value = <u8 as base::Holder<u8>>::carry::<u8>(7u8);
    let _ = *<u8 as base::Holder<u8>>::borrow(&value);
}
