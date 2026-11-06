#![feature(inherent_associated_types)]
struct S<'a>(&'a u8);impl<'a>S<'a>{type A=&'a u8;}fn f<'a>(x:S<'a>::A)->&'a u8{x}
