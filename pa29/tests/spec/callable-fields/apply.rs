#![feature(no_core)]
#![no_core]
extern crate base;
fn dbl(x: u8) -> u8 { match x { 21 => 42, _ => 7 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::apply_field(&base::Apply { f: dbl }, 21) { 42 => 42, _ => 7 } }
