#![feature(no_core)]
#![no_core]
extern crate base;
fn bad(value:&[u8]) {let _=base::identity::<[u8]>(*value);}
