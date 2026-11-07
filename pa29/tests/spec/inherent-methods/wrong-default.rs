#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match base::Opt::Some(5u8).unwrap_or(1u16) { 5 => 42, _ => 7 }
}
