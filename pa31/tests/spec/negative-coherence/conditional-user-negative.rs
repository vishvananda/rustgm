#![feature(negative_impls)]trait A{}trait Bound{}struct S<T>(T);impl<T:Bound>!A for S<T>{}fn main(){}
