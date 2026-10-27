#![feature(repr_simd)]
#![allow(dead_code)]
#[repr(simd)] pub struct V(pub [f32;4]);
#[unsafe(no_mangle)]pub extern "C" fn definition(x:V)->V{x}
