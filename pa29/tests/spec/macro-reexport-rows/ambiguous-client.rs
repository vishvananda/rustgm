#![feature(no_core)]
#![no_core]
extern crate exporter;
use exporter::builtin::*;
use exporter::alternate::*;
pub fn named() -> u32 {
    // Two globs bind `file` to two definitions that share one expander.
    let _file: &str = file!();
    1
}
