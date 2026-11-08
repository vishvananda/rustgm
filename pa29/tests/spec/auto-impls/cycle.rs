#![feature(no_core)]
#![no_core]
extern crate base;
fn need<T: base::Send>() {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<base::List>(); 42 }
