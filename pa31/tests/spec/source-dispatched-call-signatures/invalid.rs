#![feature(no_core)]
#![no_core]
extern crate base;
pub fn missing_context()->u32 {base::unsafe_scope(1)}
