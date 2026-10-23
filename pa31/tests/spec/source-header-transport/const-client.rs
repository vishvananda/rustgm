#![feature(no_core)]
#![no_core]
extern crate base;
const N:u8=base::identity(7u8);
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8 {N}
