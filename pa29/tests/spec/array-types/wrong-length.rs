#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let _a: [u8; 3] = base::keep([42, 0, 0, 0]); 42 }
