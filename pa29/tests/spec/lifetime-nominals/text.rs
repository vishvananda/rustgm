#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let t = base::Text::Word("w"); if base::is_word(&t) { 42 } else { 7 } }
