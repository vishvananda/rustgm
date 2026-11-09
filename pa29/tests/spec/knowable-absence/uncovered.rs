#![feature(no_core)]
#![no_core]
extern crate base;
trait Mine {}
impl<T: base::Error> Mine for T {}
impl<T> Mine for &T {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
