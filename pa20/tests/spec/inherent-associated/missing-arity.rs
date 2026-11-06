#![feature(inherent_associated_types)]
struct S;impl S{type A<T>=T;}fn f(_:S::A){}
