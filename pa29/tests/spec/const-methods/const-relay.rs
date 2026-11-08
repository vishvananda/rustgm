#![feature(no_core)]
#![no_core]
extern crate base;
const N: u8 = base::Held::relay("hi");
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match N { 42 => 42, _ => 7 } }
