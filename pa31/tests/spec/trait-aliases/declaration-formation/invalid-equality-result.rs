#![feature(trait_alias)]
trait P { type Out; }
struct NoCopy(u8);
struct Need<T: Copy>(T);
trait A = P<Out = Need<NoCopy>>;
fn main() {}
