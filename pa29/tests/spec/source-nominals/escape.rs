#![feature(no_core)]
#![no_core]
extern crate base;
pub fn invalid() -> base::Packet<&'static i32> { let local = 7i32; base::make(&local) }
