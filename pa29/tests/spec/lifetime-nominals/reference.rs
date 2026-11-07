#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let v = 42u8; let r = base::new_ref(&v); match *base::get(r) { 42 => 42, _ => 7 } }
