#![feature(no_core)]
#![no_core]
extern crate base;
struct Local;
impl base::DerefMutLike for &Local { fn exclusive(&mut self) -> u8 { 7 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
