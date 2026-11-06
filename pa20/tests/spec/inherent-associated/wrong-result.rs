#![feature(inherent_associated_types)]
struct S<T>(T);impl<T>S<T>{type A=u8;}fn f(x:S<u16>::A)->u16{x}
