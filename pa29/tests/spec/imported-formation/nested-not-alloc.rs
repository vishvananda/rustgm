#![feature(no_core)]
#![no_core]
extern crate base;
struct NotAlloc;
fn hold(_n: base::Nested<NotAlloc>) {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
