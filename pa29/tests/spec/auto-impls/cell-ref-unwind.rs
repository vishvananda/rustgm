#![feature(no_core)]
#![no_core]
extern crate base;
fn need<T: base::UnwindSafe>() {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<&base::Cell<u8>>(); 42 }
