#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::exact(42) { 42 => 42, _ => base::panicking::panic("unexpected") } }
