#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match base::Slot::new(1u16).peek() { 1 => 42, _ => 7 }
}
