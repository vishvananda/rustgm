#![feature(no_core)]
#![no_core]
extern crate base;
struct Mine;
impl base::Send for Mine {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
