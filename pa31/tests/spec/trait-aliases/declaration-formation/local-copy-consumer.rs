#![feature(trait_alias)]
trait A = Copy;
fn twice<T: A>(value: T) -> (T, T) { (value, value) }
fn value() -> u8 { let (left, right) = twice(7u8); left + right }
fn main() {}
