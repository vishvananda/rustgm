#![feature(no_core)]
#![no_core]
extern crate base;
fn shrink<'a, 'b: 'a>(x: base::Arguments<'b>, _y: &'a u8) -> base::Arguments<'a> { x }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let v = 1u8; match base::take(shrink(base::Arguments { text: "s" }, &v)) { 42 => 42, _ => 7 } }
