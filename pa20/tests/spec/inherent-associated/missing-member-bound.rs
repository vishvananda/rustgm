#![feature(inherent_associated_types)]
struct S;impl S{type A<T:Copy>=T;}fn f<T>(x:S::A<T>){}
