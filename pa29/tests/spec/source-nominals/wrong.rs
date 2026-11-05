#![feature(no_core)]
#![no_core]
extern crate base;
pub fn invalid() -> base::Packet<u8> { base::make(7i32) }
