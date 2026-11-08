#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::accept(base::Unit) { 42 => match base::unit() { base::Unit => 42 }, _ => 7 } }
