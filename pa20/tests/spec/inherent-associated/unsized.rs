#![feature(inherent_associated_types)]
struct S;impl S{type A=[u8];}fn f(x:&S::A)->&[u8]{x}
