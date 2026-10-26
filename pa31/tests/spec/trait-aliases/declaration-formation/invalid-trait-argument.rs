#![feature(trait_alias)]
trait P<T> {}
struct NoCopy(u8);
struct Need<T: Copy>(T);
trait A = P<Need<NoCopy>>;
fn main() {}
