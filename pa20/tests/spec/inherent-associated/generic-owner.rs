#![feature(inherent_associated_types)]
struct S<T>(T);impl<T>S<T>{type A=T;}fn f(x:S<u8>::A)->u8{x}
