#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let x: u16 = base::only(base::one(42u8));
    match x { 42 => 42, _ => 7 }
}
