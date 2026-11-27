#![feature(no_core, asm_native_fixture)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let mut x: u32 = 0; let p = &raw mut x; unsafe { base::store(p); base::bump(p); } x as i32 }
