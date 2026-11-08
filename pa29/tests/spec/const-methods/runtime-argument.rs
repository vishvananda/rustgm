#![feature(no_core)]
#![no_core]
extern crate base;
pub fn pick(x: u8) -> u8 { const N: u8 = base::Plain::new(x).get(); N }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match pick(42) { _ => 7 } }
