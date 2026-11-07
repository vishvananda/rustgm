#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::swap(true, base::Packet { value: 21i32 }, base::Packet { value: 22i32 }).first.value }
