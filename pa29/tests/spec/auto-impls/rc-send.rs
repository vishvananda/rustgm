#![feature(no_core)]
#![no_core]
extern crate base;
fn need<T: base::Send>() {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<base::Rc<u8, base::Global>>(); 42 }
