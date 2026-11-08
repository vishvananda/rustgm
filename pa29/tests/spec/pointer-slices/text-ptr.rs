#![feature(no_core)]
#![no_core]
extern crate base;
fn relay(p: *const str) -> *const str { base::text_ptr(p) }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
