#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let p = base::text(base::Arguments { text: "hi" }) as *const str as *const u8; match unsafe { *p } { 104 => 42, _ => 7 } }
