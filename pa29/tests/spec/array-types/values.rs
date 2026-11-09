#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let a = base::keep([42, 1, 2, 3]); let _e: [u8; 0] = base::empty([]); let p = &raw const a as *const u8; match unsafe { *p } { 42 => 42, _ => 7 } }
