#![feature(no_core)]
#![no_core]
#[macro_use]
extern crate prov;
pub fn spelled() -> u32 {
    let _text: &str = stringify!(a + b);
    line!()
}
