#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::apply_ptr(base::id::<u8>, 42u8) as i32 }
