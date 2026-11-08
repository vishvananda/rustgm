#![feature(no_core)]
#![no_core]
extern crate base;
fn need<T: base::Iterator + ?base::Sized>() {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<[u8]>(); 42 }
