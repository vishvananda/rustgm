#![feature(no_core)]
#![no_core]
extern crate panics;
// Before 2021, a call site of the builtin `unreachable` selects
// `unreachable_2015` from its own edition.
#[unsafe(no_mangle)]
pub extern "C" fn unreached() -> u32 {
    panics::unreachable!()
}
#[unsafe(no_mangle)]
pub extern "C" fn unreached_formatted() -> u32 {
    panics::unreachable!("at {}", 3)
}
