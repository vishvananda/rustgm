#![feature(no_core)]
#![no_core]
extern crate base;
fn wide(x: u8) -> i32 { x as i32 }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::apply(wide, 41u8) as i32 }
