#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait A{}trait Alias=A;impl Alias for u8{}fn main(){}
