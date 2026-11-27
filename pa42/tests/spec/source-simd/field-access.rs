#![feature(repr_simd)]
#![allow(dead_code)]
#[repr(simd)] pub struct V(pub [f32;4]);
pub fn fields(x:&mut V)->&mut[f32;4]{&mut x.0} pub fn first(x:&V)->f32{x.0[0]}
