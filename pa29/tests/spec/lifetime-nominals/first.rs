#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let a = base::first(base::Arguments { text: "x" }, base::Arguments { text: "y" }); match base::take(a) { 42 => 42, _ => 7 } }
