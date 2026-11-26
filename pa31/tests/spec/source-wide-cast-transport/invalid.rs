#![feature(no_core,f16)]
#![no_core]
extern crate base;
pub fn invalid(value: f16) -> bool { value as bool }
