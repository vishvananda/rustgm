#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let mut x = 7u8; let p = base::same_mut(&mut x as *mut u8); unsafe { *p = 42 }; match x { 42 => 42, _ => 7 } }
