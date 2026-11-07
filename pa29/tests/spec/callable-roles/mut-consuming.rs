#![feature(no_core)]
#![no_core]
extern crate base;
pub struct Token(pub u8);
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let token = Token(42);
    base::apply_mut(move |_x| { let moved = token; moved.0 }, 0u8) as i32
}
