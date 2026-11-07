#![feature(no_core)]
#![no_core]
extern crate exporter;
pub fn joined() -> u32 {
    // The definition's own module is private; only its re-exports are public.
    let _text: &str = exporter::hidden::concat!("a", "b");
    1
}
