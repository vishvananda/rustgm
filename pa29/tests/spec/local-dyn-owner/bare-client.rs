#![feature(no_core)]
#![no_core]
extern crate base;
pub fn accept(_: &(dyn base::Open+'static)){}
