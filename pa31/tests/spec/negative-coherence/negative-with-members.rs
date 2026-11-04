#![feature(negative_impls)]
trait A{fn f();}impl !A for u32{fn f(){}}fn main(){}
