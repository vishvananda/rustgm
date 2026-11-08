#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let x = 42u8; let r = base::Raw { p: &x as *const u8, n: 1 }; match unsafe { *r.p } { 42 => 42, _ => 7 } }
