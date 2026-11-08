#![feature(no_core)]
#![no_core]
extern crate base;
const H: base::Held<'static> = base::Held::from_str("hi");
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match H.count() { 42 => 42, _ => 7 } }
