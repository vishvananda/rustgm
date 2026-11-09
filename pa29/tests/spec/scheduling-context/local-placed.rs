#![feature(no_core)]
#![no_core]
extern crate base;
const fn local() -> usize { let x: usize = 4; x }
const COUNT: usize = local();
fn take(_: &[u8; COUNT]) -> i32 { 42 }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { take(&[1, 2, 3, 4]) }
