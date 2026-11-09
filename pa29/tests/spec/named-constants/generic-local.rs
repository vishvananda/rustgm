#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::generic_inner(&7_u8) as i32 }
