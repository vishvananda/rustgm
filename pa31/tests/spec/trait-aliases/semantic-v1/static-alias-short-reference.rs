#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait A=where Self:'static;fn need<T:A>(){}fn run<'a>(_:&'a u8){need::<&'a u8>();}fn main(){}
