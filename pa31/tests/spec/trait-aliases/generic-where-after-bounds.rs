#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait Read<V>{}trait Alias<V> = Read<V> where V:Copy;fn main(){}
