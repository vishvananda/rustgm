#![allow(dead_code)]
#![feature(trait_alias)]
struct NoCopy(u8);
trait A<T: Copy = NoCopy> = where T: Copy;
fn main() {}
