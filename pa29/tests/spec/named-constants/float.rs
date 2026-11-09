#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match (base::HALF as u8, base::QUARTER as u8) { (42, 42) => 42, _ => 1 } }
