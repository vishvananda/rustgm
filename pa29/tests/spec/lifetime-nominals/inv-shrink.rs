#![feature(no_core)]
#![no_core]
extern crate base;
fn shrink<'a, 'b: 'a>(x: base::Inv<'b>, _y: &'a u8) -> base::Inv<'a> { x }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let x = 42u8; let mut v = &x; match base::read(shrink(base::Inv { slot: &mut v }, &x)) { 42 => 42, _ => 7 } }
