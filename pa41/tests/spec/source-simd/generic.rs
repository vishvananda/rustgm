#![feature(repr_simd)]
#![allow(dead_code)]
#[repr(simd)]pub struct Generic<T,const N:usize>(pub[T;N]);pub fn identity<T,const N:usize>(x:Generic<T,N>)->Generic<T,N>{x}
