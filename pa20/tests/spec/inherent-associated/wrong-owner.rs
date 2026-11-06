#![feature(inherent_associated_types)]
struct S;impl S{type A=u8;}fn f(_:<u8>::A){}
