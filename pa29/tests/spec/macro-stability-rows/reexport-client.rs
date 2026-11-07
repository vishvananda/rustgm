#![feature(no_core)]
#![no_core]
extern crate gates;
use gates::reexported::answer;
pub fn answer() -> u32 {
    answer!()
}
