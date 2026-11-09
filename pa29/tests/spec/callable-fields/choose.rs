#![feature(no_core)]
#![no_core]
extern crate base;
fn inc(x: u8) -> i32 { match x { 41 => 42, _ => 7 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::choose(&base::Choice::Computed(inc), 41) }
