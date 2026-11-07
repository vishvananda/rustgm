#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { if base::exact(8u8, true, false) { 1 } else { 23 } }
