#![allow(unconditional_panic)]
struct S;impl S{const A:&'static u8=&(1/0);}fn main(){}