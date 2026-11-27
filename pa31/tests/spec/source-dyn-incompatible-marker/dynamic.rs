#![feature(no_core)]
#![no_core]
extern crate base;

pub fn dynamic(_: &dyn base::Pure) {}
