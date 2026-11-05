#![feature(no_core)]
#![no_core]
extern crate base;
pub fn invalid() -> i32 { base::take(base::hide(7i32)) }
