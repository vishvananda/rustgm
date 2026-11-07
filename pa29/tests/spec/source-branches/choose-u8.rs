#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::choose(false, false, 5u8, 6u8, 13u8) as i32 }
