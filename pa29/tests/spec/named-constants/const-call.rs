#![feature(no_core)]
#![no_core]
extern crate base;
const COUNT: usize = base::lane_count();
fn take(_: &[u8; COUNT]) -> i32 { 42 }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { take(&[1, 2, 3, 4]) }
