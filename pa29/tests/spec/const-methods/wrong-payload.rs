#![feature(no_core)]
#![no_core]
extern crate base;
const N: u8 = base::Plain::new(1u16).get();
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match N { _ => 7 } }
