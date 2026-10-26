#![feature(trait_alias)]
trait P<T> {}
struct Need<'a, T: 'a>(&'a T);
trait A<'a, T: 'a> = P<Need<'a, T>>;
fn main() {}
