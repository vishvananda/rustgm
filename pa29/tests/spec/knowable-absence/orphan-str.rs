#![feature(no_core)]
#![no_core]
extern crate base;
impl base::Error for str {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
