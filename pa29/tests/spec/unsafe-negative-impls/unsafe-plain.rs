#![feature(no_core)]
#![no_core]
extern crate base;
struct Mine;
unsafe impl base::Plain for Mine {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
