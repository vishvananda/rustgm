#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::Wrap(4, 2).0 { 42 => 42, _ => 7 } }
