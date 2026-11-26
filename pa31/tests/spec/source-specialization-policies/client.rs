#![feature(no_core)]
#![no_core]

extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn applicable_blanket() -> u8 {
    <[u16; 3] as base::Compared>::answer()
}

#[unsafe(no_mangle)]
pub extern "C" fn applicable_specialized() -> u8 {
    <[u8; 2] as base::Compared>::answer()
}

#[unsafe(no_mangle)]
pub extern "C" fn trusted_blanket() -> u8 {
    <[u16; 3] as base::Ordered>::answer()
}

#[unsafe(no_mangle)]
pub extern "C" fn trusted_specialized() -> u8 {
    <[u8; 2] as base::Ordered>::answer()
}
