#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::choose(true, false, base::Packet { value: 1i32 }, base::Packet { value: 2i32 }, base::Packet { value: 3i32 }).value }
