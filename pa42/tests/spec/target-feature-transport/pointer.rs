#![feature(no_core, asm_native_fixture)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let f: unsafe fn(u32, u32) -> u32 = base::second; unsafe { f(0, 42) as i32 } }
