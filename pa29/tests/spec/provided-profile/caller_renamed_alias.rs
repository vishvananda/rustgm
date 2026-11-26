#![feature(no_core)]
#![no_core]

extern crate renamed_base;

unsafe extern "C" {
    fn observe(value: u8);
}

fn main() {
    let value = <u8 as renamed_base::Holder<u8>>::carry::<u8>(7u8);
    let borrowed = <u8 as renamed_base::Holder<u8>>::borrow(&value);
    unsafe {
        observe(*borrowed);
    }
}
