#![feature(no_core)]
#![no_core]
extern crate base;
static C: base::Cell<u8> = base::Cell { value: 42 };
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match C.value { 42 => 42, _ => 7 } }
