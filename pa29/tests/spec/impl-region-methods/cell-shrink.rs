#![feature(no_core)]
#![no_core]
extern crate base;
fn shrink<'s, 'l: 's>(c: base::Cell<'l>, v: &'s u8) -> u8 { c.set(v) }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let x = 1u8; let mut r: &u8 = &x; let y = 42u8; shrink(base::Cell { slot: &mut r }, &y) as i32 }
