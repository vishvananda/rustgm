#![feature(no_core)]
#![no_core]
extern crate base;
fn width(_: &[u8; base::LANES]) -> i32 { 42 }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { width(&[1, 2, 3]) }
