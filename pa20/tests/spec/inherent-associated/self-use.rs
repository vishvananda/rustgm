#![feature(inherent_associated_types)]
struct S;impl S{type A=u8;fn f(x:Self::A)->Self::A{x}}
