#![feature(no_core)]
#![no_core]
extern crate base;

const WRONG: fn(u8) -> bool = base::current().message;
