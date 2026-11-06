#![feature(inherent_associated_types)]
struct S<T>(T);impl S<u8>{type A=u8;}impl S<u16>{type A=u16;}fn f(x:S<u16>::A)->u16{x}
