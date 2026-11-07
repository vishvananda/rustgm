#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match base::peek(base::some(42u8)) { 42 => match base::peek(base::none()) { 0 => 42, _ => 8 }, _ => 7 }
}
