#![feature(no_core)]
#![no_core]
extern crate base;
const N: usize = base::id(4usize);
fn take(_: &[u8; N]) -> i32 { 42 }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { take(&[1, 2, 3, 4]) }
