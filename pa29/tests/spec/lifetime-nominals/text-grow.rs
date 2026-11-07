#![feature(no_core)]
#![no_core]
extern crate base;
fn grow<'a, 'b: 'a>(x: base::Text<'a>) -> base::Text<'b> { x }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { if base::is_word(&grow(base::Text::Word("w"))) { 42 } else { 7 } }
