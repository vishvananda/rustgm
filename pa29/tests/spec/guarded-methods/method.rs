#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let l = base::Lane(42); unsafe { l.get() as i32 } }
