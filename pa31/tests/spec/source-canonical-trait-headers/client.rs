#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn answer() -> u8 {
    base::identity(57u8)
}
