#![feature(no_core, gated_answer)]
#![no_core]
extern crate gates;
pub fn answer() -> u32 {
    gates::answer!()
}
