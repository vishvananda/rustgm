#![feature(inherent_associated_types)]
struct S;impl S{type A:Copy=u8;}fn f(x:S::A)->u8{x}
