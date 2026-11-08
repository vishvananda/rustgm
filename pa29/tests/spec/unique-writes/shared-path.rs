#![feature(no_core)]
#![no_core]
extern crate base;
fn poke(c: &base::Count<'_>) -> u8 { *c.n = 1; 42 }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let mut n = 7u8; poke(&base::Count { n: &mut n }) as i32 }
