#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let o = base::Opt::Some(5u8);
    let a = o.unwrap_or(1u8);
    match o.unwrap_or(2u8) { 5 => a as i32, _ => 7 }
}
