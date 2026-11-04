#![feature(negative_impls)]
trait A:B{}trait B{}impl !A for u32{}fn main(){}
