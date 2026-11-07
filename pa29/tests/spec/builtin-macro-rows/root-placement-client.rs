#![feature(no_core)]
#![no_core]
extern crate prov;
pub fn joined() -> u32 {
    // `concat` is placed in `prov::builtin`, never at the provider root.
    let _text: &str = prov::concat!("a", "b");
    1
}
