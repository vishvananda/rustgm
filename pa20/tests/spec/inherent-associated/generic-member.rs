#![feature(inherent_associated_types)]
struct Wrap<T>(T);struct S;impl S{type A<T>=Wrap<T>;}fn f(x:S::A<u8>)->Wrap<u8>{x}
