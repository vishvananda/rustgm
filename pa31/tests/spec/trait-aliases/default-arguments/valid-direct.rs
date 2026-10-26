#![allow(dead_code)]
#![feature(trait_alias)]
trait A<T: Copy = u8> = where T: Copy;
fn main() {}
