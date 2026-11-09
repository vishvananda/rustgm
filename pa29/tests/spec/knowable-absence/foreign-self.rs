#![feature(no_core)]
#![no_core]
extern crate base;
trait Mine {}
impl<T: base::Error> Mine for T {}
impl Mine for base::Counter {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
