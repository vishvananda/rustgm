#![feature(no_core)]
#![no_core]
extern crate base;
fn answer(x: u8) -> u8 { match x { 41 => 42, _ => 7 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::apply_ref(answer, 41u8) as i32 }
