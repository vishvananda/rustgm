#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::into_inner(base::cell(42u8)) { 42 => 42, _ => 7 } }
