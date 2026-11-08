#![feature(no_core)]
#![no_core]
extern crate base;
fn grow<'a, 'b: 'a>(x: base::Marked<&'a u8>) -> base::Marked<&'b u8> { x }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::marked_value(grow(base::marked(42))) { 42 => 42, _ => 7 } }
