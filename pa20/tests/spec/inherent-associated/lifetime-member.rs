#![feature(inherent_associated_types)]
struct S;impl S{type A<'a>=&'a u8;}fn f<'a>(x:S::A<'a>)->&'a u8{x}
