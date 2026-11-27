#![feature(repr_simd,simd_ffi)]
#![allow(dead_code)]
#[repr(simd)] pub struct V(pub [f32;4]);
unsafe extern "C"{pub fn foreign(x:V)->V;}
