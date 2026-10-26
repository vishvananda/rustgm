#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait A<T:?Sized>=;fn run<S:A<str>>(){}fn main(){}
