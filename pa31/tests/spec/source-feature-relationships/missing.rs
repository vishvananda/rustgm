#![feature(no_core)]
#![no_core]
extern crate base;

pub fn read() -> u8 { base::dependent() }
