#![feature(no_core)]
#![no_core]
extern crate base;
trait Mark {}
trait Mine {}
impl<T: Mark> Mine for T {}
impl<T> Mine for &T {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
