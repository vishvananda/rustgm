#![feature(no_core)]
#![no_core]
extern crate base;
fn need<T: base::Send>() {}
struct W<T> { v: T }
unsafe impl base::Send for W<u8> {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<W<u16>>(); 42 }
