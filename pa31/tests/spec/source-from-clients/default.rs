#![feature(no_core)]
#![no_core]
extern crate minicore;
#[unsafe(no_mangle)]
pub extern "C" fn imported_observed() -> u8 {
    let value: f32 = minicore::Into::into(1.0);
    value as u8
}
