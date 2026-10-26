#![allow(dead_code)]
#![feature(trait_alias)]
struct NoCopy(u8);
trait Leaf<T> = where T: Copy;
trait Middle<T> = Leaf<T>;
trait Outer<T = NoCopy> = Middle<T>;
fn main() {}
