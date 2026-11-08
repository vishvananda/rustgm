#![feature(no_core)]
#![no_core]
extern crate base;
fn need<T: base::Send>() {}
struct Mine { c: base::Cell<u8> }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<Mine>(); 42 }
