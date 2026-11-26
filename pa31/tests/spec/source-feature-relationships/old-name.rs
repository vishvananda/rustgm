#![feature(no_core)]
#![no_core]
#![feature(former_api)]
extern crate base;

pub fn read() -> u8 { base::renamed() }
