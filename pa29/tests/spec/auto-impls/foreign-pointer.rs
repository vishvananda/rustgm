#![feature(no_core)]
#![no_core]
extern crate base;
struct Mine;
unsafe impl base::Send for *const Mine {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
