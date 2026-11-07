#![feature(no_core)]
#![no_core]
extern crate panics;
// A 2015 call site of the builtin `panic` selects `panic_2015`, which stays
// rejected by name.
pub fn legacy() -> u32 {
    panics::panic!()
}
