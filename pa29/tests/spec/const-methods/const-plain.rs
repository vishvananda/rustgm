#![feature(no_core)]
#![no_core]
extern crate base;
const P: base::Plain = base::Plain::new(42);
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match P.get() { 42 => 42, _ => 7 } }
