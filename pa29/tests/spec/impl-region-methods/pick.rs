#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let a = base::Held::new("a", 42); let b = base::Held::new("b", 7); match a.pick(&b).count() { 42 => 42, _ => 7 } }
