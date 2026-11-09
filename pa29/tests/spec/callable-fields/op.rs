#![feature(no_core)]
#![no_core]
extern crate base;
fn add(a: u8, b: u8) -> i32 { match a { 40 => match b { 2 => 42, _ => 7 }, _ => 7 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::op_apply(&base::Op(add), 40, 2) }
