#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let s = base::Slot::new(40u8);
    match s.peek() { 40 => match base::Slot::<u8>::get(s) { 40 => 42, _ => 8 }, _ => 7 }
}
