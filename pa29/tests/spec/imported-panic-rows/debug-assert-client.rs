#![feature(no_core)]
#![no_core]
extern crate panics;
pub fn checked(flag: bool) -> u32 {
    panics::debug_assert!(flag, "flag");
    7
}
