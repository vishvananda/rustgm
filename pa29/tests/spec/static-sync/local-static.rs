#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { static N: u8 = 42; match N { 42 => 42, _ => 7 } }
