#![feature(no_core)]
#![no_core]
extern crate base;
const V: u8 = unsafe { base::Plain { n: 1 }.value() };
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { V as i32 }
