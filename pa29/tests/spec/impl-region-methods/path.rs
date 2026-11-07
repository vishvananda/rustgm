#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let h = base::Held::new("p", 42); match base::Held::count(&h) { 42 => 42, _ => 7 } }
