#![feature(negative_impls)]
unsafe trait A{}impl !A for u32{}fn main(){}
