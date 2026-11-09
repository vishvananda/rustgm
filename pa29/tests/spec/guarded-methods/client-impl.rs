#![feature(no_core)]
#![no_core]
extern crate base;
struct Mine;
impl base::Get for Mine { unsafe fn fetch(&self) -> u8 { 42 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::via_get(&Mine) as i32 }
