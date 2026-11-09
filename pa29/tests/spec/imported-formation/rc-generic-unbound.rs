#![feature(no_core)]
#![no_core]
extern crate base;
fn hold<A>(_r: base::Rc<u8, A>) {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
