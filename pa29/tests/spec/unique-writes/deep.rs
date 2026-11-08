#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let mut n = 7u8; let mut c = base::Count { n: &mut n }; match base::deep(base::Outer { inner: &mut c }) { 42 => 42, _ => 7 } }
