#![feature(no_core, asm_native_fixture)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let mut out = base::V([0, 0, 0, 0]); unsafe { base::put(&raw mut out, [42, 1, 2, 3]) }; 42 }
