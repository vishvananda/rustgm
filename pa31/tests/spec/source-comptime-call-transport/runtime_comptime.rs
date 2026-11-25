#![feature(no_core)]
#![no_core]
extern crate base;

pub fn runtime_comptime() -> u32 { base::chained() }
