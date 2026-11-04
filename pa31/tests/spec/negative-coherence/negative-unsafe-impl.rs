#![feature(negative_impls)]
unsafe trait A{}unsafe impl !A for u32{}fn main(){}
