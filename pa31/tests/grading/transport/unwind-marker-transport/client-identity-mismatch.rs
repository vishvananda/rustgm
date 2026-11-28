#![feature(no_core)]
#![no_core]

extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn read_owned() -> u8 {
    base::require(&base::Shared(5))
}

#[unsafe(no_mangle)]
pub extern "C" fn read_shared() -> u8 {
    base::require(&&base::Shared(6))
}

#[unsafe(no_mangle)]
pub extern "C" fn read_shared_safe() -> u8 {
    base::require_shared(&base::Shared(9))
}
