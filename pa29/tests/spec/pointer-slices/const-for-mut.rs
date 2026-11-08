#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let x = 42u8; let _ = base::same_mut(&x as *const u8); 7 }
