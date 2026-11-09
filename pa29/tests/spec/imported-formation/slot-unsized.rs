#![feature(no_core)]
#![no_core]
extern crate base;
fn hold(_s: &base::Slot<str>) {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
