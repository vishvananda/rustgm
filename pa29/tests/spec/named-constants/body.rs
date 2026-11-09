#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match (base::lanes(), base::neg() as u8) { (4, 255) => 42, _ => 1 } }
