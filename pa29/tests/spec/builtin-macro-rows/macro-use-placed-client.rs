#![feature(no_core)]
#![no_core]
#[macro_use]
extern crate prov;
pub fn joined() -> u32 {
    // `concat` is placed in `prov::builtin`, so the macro-use prelude lacks it.
    let _text: &str = concat!("a", "b");
    1
}
