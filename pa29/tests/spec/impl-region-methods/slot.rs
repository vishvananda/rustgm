#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let v = 42u8; let s = base::Slot::new(&v); match *s.peek_ref() { 42 => 42, _ => 7 } }
