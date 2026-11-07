#![feature(no_core)]
#![no_core]
#[macro_use]
extern crate exporter;
pub fn spelled() -> u32 {
    // Root re-exports enter the macro-use prelude with the root definitions.
    let _file: &str = file!();
    let _text: &str = spelled!(a + b);
    line!()
}
