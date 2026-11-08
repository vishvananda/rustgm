#![feature(no_core)]
#![no_core]
extern crate base;
fn need<T: base::Send>() {}
struct W<T> { v: T, p: *const u8 }
unsafe impl base::Send for W<u8> {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<W<u8>>(); 42 }
