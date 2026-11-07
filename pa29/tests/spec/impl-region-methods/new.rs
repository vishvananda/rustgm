#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let h = base::Held::new("hi", 42); match h.count() { 42 => 42, _ => 7 } }
