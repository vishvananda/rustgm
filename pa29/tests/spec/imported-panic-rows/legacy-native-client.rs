#![feature(no_core)]
#![no_core]
extern crate panics;
// `plain_fail` has no `edition_panic`, so its own 2024 edition selects 2021.
#[unsafe(no_mangle)]
pub extern "C" fn unskipped() -> u32 {
    panics::plain_fail!()
}
