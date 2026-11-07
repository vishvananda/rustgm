#![feature(no_core)]
#![no_core]
extern crate base;
fn keep<'a>(x: base::Inv<'a>) -> base::Inv<'a> { x }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let x = 42u8; let mut v = &x; match base::read(keep(base::Inv { slot: &mut v })) { 42 => 42, _ => 7 } }
