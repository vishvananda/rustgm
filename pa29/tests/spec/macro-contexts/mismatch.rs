#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let b: bool = base::relay(); match b { true => 42, false => 7 } }
