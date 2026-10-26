#![feature(trait_alias)]
trait P<T> {}
struct NoCopy(u8);
struct Need<T: Copy>(T);
trait Base<T> = P<T>;
trait A = Base<Need<NoCopy>>;
fn main() {}
