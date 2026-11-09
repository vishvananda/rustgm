#![feature(no_core, asm_native_fixture)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let mut out = [0u32, 0, 0, 0]; let p = &raw mut out as *mut base::V; unsafe { base::put(p, base::V([42, 1, 2, 3])); *(p as *mut u32) as i32 } }
