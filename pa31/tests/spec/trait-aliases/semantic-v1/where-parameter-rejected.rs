#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait P{}impl P for u8{}trait A<T> = where T:P;fn require<T:P>()->u8{7}fn run<S:A<T>,T>()->u8{require::<T>()}fn main(){let _=run::<(),u16>();}
