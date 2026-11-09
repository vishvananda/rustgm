#![feature(no_core)]
#![no_core]
extern crate base;
impl<T> base::Error for T {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
