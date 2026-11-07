#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let t; { let h = base::Held::new("hi", 1); t = h.text(); } let p = t as *const str as *const u8; match unsafe { *p } { 104 => 42, _ => 7 } }
