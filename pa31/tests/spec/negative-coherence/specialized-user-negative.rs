#![feature(negative_impls)]trait A{}struct S<T>(T);impl !A for S<u32>{}fn main(){}
