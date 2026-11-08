#![feature(no_core)]
#![no_core]
extern crate base;
fn need<T: base::Sync>() {}
struct Holder { inner: base::Pair<base::Raw> }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<Holder>(); 42 }
