#![feature(no_core)]
#![no_core]

extern crate base;

fn main() {
    let borrowed;
    {
        let value = 7u8;
        borrowed = <u8 as base::Holder<u8>>::borrow(&value);
    }
    let _ = *borrowed;
}
