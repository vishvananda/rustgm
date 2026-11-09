#![feature(no_core)]
#![no_core]
extern crate base;
fn hold(_r: base::Rc<u8, base::Global>) {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
