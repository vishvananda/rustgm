#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match base::code(base::high()) { 39 => match base::code(base::low()) { 3 => 42, _ => 8 }, _ => 7 }
}
