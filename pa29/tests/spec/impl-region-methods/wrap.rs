#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let h = base::Held::new("w", 42); let i = base::wrap(&h); match (i.flag, i.held.count()) { (true, 42) => 42, _ => 7 } }
