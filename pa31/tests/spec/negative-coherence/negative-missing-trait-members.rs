#![feature(negative_impls)]
trait A{fn f();type X;const N:u8;}impl !A for u32{}fn main(){}
