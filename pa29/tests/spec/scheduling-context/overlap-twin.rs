#![feature(no_core)]
#![no_core]
extern crate base;
pub struct Local;
impl base::Marker for Local {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
