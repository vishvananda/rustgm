#![feature(no_core)]
#![no_core]
extern crate base;
fn swap<'x>(c: base::Count<'x>, m: &'x mut u8) -> u8 { c.n = m; 42 }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let mut n = 7u8; let mut k = 42u8; swap(base::Count { n: &mut n }, &mut k) as i32 }
