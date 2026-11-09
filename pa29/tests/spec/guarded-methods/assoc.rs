#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let l = unsafe { base::Lane::make() }; l.0 as i32 }
