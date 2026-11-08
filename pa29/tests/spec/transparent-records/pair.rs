#![feature(no_core)]
#![no_core]
extern crate base;
#[repr(transparent)]
struct Pair(base::Wrap, base::Wrap);
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
