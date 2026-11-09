#![feature(no_core)]
#![no_core]
extern crate base;
struct L;
impl base::Show for L { fn show(self) -> u8 { 42 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { base::make::<L>()(L) as i32 }
