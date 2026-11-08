#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let x = 1u8; match base::raw_n(base::raw(&x as *const u8)) { 42 => 42, _ => 7 } }
