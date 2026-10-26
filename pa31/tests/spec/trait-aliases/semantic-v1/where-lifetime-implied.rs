#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait A<'a>=where Self:'a;fn require<'a,T:'a>(){}fn run<'a,T:A<'a>>(){require::<'a,T>();}fn main(){}
