#![feature(no_core)]
#![no_core]

extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed_default() -> u8 {
    <[u8; 3] as base::Answer>::answer()
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_specialized() -> u8 {
    <[u8; 2] as base::Answer>::answer()
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_inherited_size() -> usize {
    <[u8; 2] as base::Answer>::inherited_size()
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_inherited_type() -> u16 {
    let expected: <[u8; 2] as base::Answer>::Output = 641_u16;
    expected
}
