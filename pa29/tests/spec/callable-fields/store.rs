#![feature(no_core)]
#![no_core]
extern crate base;
fn other(_v: u8) -> i32 { 7 }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let mut s = base::Slot { f: other }; base::store(&mut s, base::hold); base::via(&s) }
