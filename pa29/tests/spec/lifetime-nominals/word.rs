#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { if base::is_word(&base::word("w")) { 42 } else { 7 } }
