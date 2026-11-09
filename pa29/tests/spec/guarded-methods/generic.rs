#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let c = base::Cell(42u8); unsafe { c.read() as i32 } }
