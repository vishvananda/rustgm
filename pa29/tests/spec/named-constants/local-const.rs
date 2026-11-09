#![feature(no_core)]
#![no_core]
extern crate base;
const MINE: usize = base::LANES;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match MINE { 4 => 42, _ => 1 } }
