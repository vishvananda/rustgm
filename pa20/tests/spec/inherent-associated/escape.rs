#![feature(inherent_associated_types)]
struct S;impl S{type A<'a>=&'a u8;}fn f(x:&u8)->S::A<'static>{x}
