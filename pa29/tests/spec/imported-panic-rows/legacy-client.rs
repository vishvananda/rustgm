#![feature(no_core)]
#![no_core]
extern crate panics;
// `fail_with` is skipped, so this 2018 call site selects `panic_2015`.
pub fn legacy() -> u32 {
    panics::fail_with!()
}
