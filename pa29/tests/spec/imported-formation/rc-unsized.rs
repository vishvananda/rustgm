#![feature(no_core)]
#![no_core]
extern crate base;
fn hold(_r: &base::Rc<str, base::Global>) {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
