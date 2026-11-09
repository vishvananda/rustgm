#![feature(no_core)]
#![no_core]
extern crate base;
fn inc(v: u8) -> u8 { match v { 40 => 41, 41 => 42, _ => 7 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::twice(inc, 40) as i32 }
