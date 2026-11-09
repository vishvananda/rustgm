#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let p = base::ctor()(41); match p.0 { 41 => 42, _ => 7 } }
