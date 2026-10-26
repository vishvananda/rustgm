#![feature(trait_alias)]
trait P<T> {}
struct Need<T: Copy>(T);
trait A = P<Need<u8>>;
fn main() {}
