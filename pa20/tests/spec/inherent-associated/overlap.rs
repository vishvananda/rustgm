#![feature(inherent_associated_types)]
struct S<T>(T);impl<T:Copy>S<T>{type A=u8;}impl<T:Clone>S<T>{type A=u16;}
