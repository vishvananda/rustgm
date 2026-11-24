#![feature(no_core)]
#![no_core]
extern crate base;

const BAD: u8 = base::Signal::<u8, 4>::FLAG;
