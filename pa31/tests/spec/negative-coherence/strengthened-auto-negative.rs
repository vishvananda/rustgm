#![feature(negative_impls,auto_traits)]auto trait A{}trait Bound{}struct S<T>(T);impl<T:Bound>!A for S<T>{}fn main(){}
