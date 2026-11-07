#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::one(42u8) { base::One::Only(42) => 42, base::One::Only(_) => 7 } }
