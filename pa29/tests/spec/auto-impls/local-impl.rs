#![feature(no_core)]
#![no_core]
extern crate base;
fn need<T: base::Send>() {}
struct Holder { p: *const u8 }
unsafe impl base::Send for Holder {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<Holder>(); 42 }
