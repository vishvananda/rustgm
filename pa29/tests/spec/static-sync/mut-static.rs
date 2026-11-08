#![feature(no_core)]
#![no_core]
extern crate base;
static mut N: u8 = 42;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match unsafe { N } { 42 => 42, _ => 7 } }
