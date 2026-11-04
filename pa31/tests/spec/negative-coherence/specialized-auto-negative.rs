#![feature(negative_impls,auto_traits)]auto trait A{}struct S<T>(T);impl !A for S<u32>{}fn main(){}
