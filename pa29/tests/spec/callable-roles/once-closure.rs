#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::apply(|x| match x { 41 => 42, _ => 7 }, 41u8) as i32 }
