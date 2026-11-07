#![feature(no_core)]
#![no_core]
extern crate base;
fn grow<'a, 'b: 'a>(x: base::Arguments<'a>) -> base::Arguments<'b> { x }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::take(grow(base::Arguments { text: "g" })) { 42 => 42, _ => 7 } }
