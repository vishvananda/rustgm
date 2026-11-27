#![feature(no_core, asm_native_fixture)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let mut x: u8 = 0; unsafe { base::store(&raw mut x) }; 42 }
