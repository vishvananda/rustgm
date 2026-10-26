#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait A=;fn run<T:?Sized+A>(){}fn main(){run::<str>();}
