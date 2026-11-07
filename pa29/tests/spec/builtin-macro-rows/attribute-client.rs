#![feature(no_core)]
#![no_core]
extern crate prov;
#[prov::builtin::deeper::test]
fn removed() {}
pub fn kept() -> u32 {
    1
}
