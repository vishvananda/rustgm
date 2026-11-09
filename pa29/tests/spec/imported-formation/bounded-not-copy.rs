#![feature(no_core)]
#![no_core]
extern crate base;
struct NotCopy;
fn hold(_b: base::Bounded<NotCopy>) {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
