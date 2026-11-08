#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::take_u32(base::Wrap(42)) { 42 => 42, _ => 7 } }
