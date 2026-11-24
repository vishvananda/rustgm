#![feature(no_core)]
#![no_core]
extern crate base;

const WRONG: fn(u8, &mut base::Arguments<'_>) -> bool = base::current().message;
