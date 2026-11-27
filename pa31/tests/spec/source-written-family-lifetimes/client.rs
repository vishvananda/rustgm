#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed(value: u8) -> u8 {
    let pair: <base::Zip<u8> as base::Iter>::Item<'static> = (value, 37);
    pair.1
}
