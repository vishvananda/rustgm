#![feature(no_core)]
#![no_core]
extern crate base;

const VALUE: u8 = {
    let value = 41u8;
    unsafe { *base::borrow_pointer(&value) }
};

#[unsafe(no_mangle)]
pub extern "C" fn observed_const() -> u8 {
    VALUE
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_runtime(value: u8) -> u8 {
    unsafe { *base::borrow_pointer(&value) }
}
