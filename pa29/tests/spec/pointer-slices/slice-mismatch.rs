#![feature(no_core)]
#![no_core]
extern crate base;
fn relay(s: &[u16]) -> u8 { base::width(s) }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 7 }
