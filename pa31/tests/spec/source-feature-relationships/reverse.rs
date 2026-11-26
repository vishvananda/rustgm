#![feature(no_core)]
#![no_core]
#![feature(dependent_api)]
extern crate base;

pub fn read() -> u8 { base::controlling() }
