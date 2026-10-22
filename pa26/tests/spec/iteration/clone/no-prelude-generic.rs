#![no_implicit_prelude]
extern crate core;fn duplicate<T:core::clone::Clone>(x:&T)->T{x.clone()}fn main(){let a=[7u8];let _=duplicate(&<[u8]>::iter(&a));}
