#![feature(no_core)]
#![no_core]
extern crate exporter;
use exporter::prelude::rust_2024::*;
use exporter::builtin::*;
pub fn spelled() -> u32 {
    // `concat` and `file` reach this crate through both globs with one identity.
    let _joined: &str = concat!("a", "b");
    let _file: &str = file!();
    let _text: &str = stringify!(c);
    let _answer: u32 = forty_two!();
    line!()
}
