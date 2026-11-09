#![feature(no_core)]
#![no_core]
extern crate base;
const N: usize = base::four();
fn take(_: usize) -> i32 { 42 }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { take(N) }
