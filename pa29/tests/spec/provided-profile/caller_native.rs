#![feature(no_core)]
#![no_core]

extern crate base;

unsafe extern "C" {
    fn observe(value: u8);
}

#[unsafe(no_mangle)]
pub extern "C" fn provided_observed() -> u8 {
    let value = <u8 as base::Holder<u8>>::carry::<u8>(7u8);
    let borrowed = <u8 as base::Holder<u8>>::borrow(&value);
    *borrowed
}

fn main() {
    unsafe {
        observe(provided_observed());
    }
}
