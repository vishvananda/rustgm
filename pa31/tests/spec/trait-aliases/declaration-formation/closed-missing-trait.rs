#![feature(trait_alias)]
trait P {}
trait A = where u8: P;
fn main() {}
