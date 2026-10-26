#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(specialization)]
trait T{type A;}impl<X>T for X{default type A=u8;}fn main(){}
