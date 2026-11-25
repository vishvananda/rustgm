#![feature(no_core)]
#![no_core]
extern crate base;

pub fn wrong_result() -> bool { base::generic::<7>() }
