#![feature(no_core)]
#![no_core]
extern crate gates;
pub fn answer() -> u32 {
    let _spelled: &str = gates::stringify!(a b);
    gates::wrapper!()
}
pub fn line() -> u32 {
    gates::line_wrapper!()
}
