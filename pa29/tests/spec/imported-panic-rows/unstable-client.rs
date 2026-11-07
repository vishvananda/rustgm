#![feature(no_core)]
#![no_core]
extern crate panics;
pub fn direct() -> u32 {
    panics::panic::panic_2021!()
}
