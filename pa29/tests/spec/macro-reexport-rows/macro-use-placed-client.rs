#![feature(no_core)]
#![no_core]
#[macro_use]
extern crate exporter;
pub fn joined() -> u32 {
    // `concat` is bound in `builtin` and the prelude modules, never at the root.
    let _text: &str = concat!("a", "b");
    1
}
