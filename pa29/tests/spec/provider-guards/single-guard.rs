#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match base::only_large(base::one(2u8)) {
        0 => match base::only_large(base::one(42u8)) { 42 => 42, _ => 8 },
        _ => 7,
    }
}
