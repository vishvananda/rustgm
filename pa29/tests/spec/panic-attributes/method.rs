#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let g = base::Gauge { level: 42 }; match g.read() { 42 => 42, _ => 7 } }
