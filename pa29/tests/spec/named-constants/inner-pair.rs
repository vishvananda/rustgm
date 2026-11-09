#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match (base::first_inner(), base::second_inner(), base::nested()) { (40, 2, 42) => 42, _ => 1 }
}
