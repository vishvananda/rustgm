#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let mut total = base::bump(1i32, 40i32, 1i32).count; total += 1i32; total }
