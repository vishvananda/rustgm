#![feature(repr_simd)]
#![allow(dead_code)]
#[repr(simd)] pub struct V(pub [f32;4]);
#[repr(C)]pub struct Wrapper(V);unsafe extern "C"{pub fn foreign(x:Wrapper)->Wrapper;}
