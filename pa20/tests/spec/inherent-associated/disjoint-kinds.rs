#![feature(inherent_associated_types)]
struct S<T>(T);impl S<u8>{type A=u8;}impl S<u16>{fn A()->u16{1}}fn f(x:S<u8>::A)->u8{let _=S::<u16>::A();x}
