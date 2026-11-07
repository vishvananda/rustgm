#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let x: u8 = base::name(); match x { 42 => 42, _ => 7 } }
