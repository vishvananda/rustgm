#![no_std]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::grade(9i32, 41i32, 42i32) }
