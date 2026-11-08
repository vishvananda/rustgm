#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let mut n = 7u8; let _ = base::bump(base::Count { n: &mut n }); match n { 42 => 42, _ => 7 } }
