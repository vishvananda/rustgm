#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::grade(1i32, base::Packet { value: 1i32 }, base::Packet { value: 2i32 }).value }
