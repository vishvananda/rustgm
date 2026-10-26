#![feature(no_core)]#![no_core]
extern crate base; pub fn needs<T:base::num::Sealed>() {}
