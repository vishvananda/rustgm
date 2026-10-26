#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(min_specialization)]
trait T{const C:u8;} impl<X>T for X{default const C:u8=1;} fn main(){}
