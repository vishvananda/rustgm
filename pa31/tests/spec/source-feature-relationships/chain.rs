#![feature(no_core)]
#![no_core]
#![feature(controlling_api)]
extern crate base;

pub fn read() -> u8 { base::chained() }
