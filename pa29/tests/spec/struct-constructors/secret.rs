#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::reveal(base::secret(42)) { 42 => match base::inner::narrow(7) { _ => 42 }, _ => 7 } }
