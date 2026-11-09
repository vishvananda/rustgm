#![feature(no_core)]
#![no_core]
extern crate base;
fn need<T: base::Send + base::Sync>(_: &T) {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need(&base::Slot { f: base::hold }); 42 }
