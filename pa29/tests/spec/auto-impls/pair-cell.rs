#![feature(no_core)]
#![no_core]
extern crate base;
fn need<T: base::RefUnwindSafe>() {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<base::Pair<base::Cell<u8>>>(); 42 }
