#![feature(no_core)]
#![no_core]
extern crate base;
fn hold(_b: base::Bounded<u8>) {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
