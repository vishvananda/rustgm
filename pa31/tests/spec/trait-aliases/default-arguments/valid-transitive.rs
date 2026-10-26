#![allow(dead_code)]
#![feature(trait_alias)]
trait Leaf<T> = where T: Copy;
trait Middle<T> = Leaf<T>;
trait Outer<T = u8> = Middle<T>;
fn main() {}
