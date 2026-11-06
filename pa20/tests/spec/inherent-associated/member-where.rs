#![feature(inherent_associated_types)]
struct S;impl S{type A<T> where T:Copy=T;}fn f(x:S::A<u8>)->u8{x}
