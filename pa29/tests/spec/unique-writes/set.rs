#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let x = 7u8; let y = 42u8; let mut r: &u8 = &x; match (base::Cell { slot: &mut r }).set(&y) { 42 => 42, _ => 7 } }
