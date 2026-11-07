#![feature(no_core)]
#![no_core]
extern crate gates;
pub fn line() -> u32 {
    gates::line!()
}
