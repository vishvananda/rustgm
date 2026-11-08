#![feature(no_core)]
#![no_core]
extern crate base;
pub fn peek(n: base::NonNull<u8>) -> i32 { let _p = n.pointer; 7 }
