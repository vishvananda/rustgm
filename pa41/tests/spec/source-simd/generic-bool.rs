#![feature(repr_simd)]
#![allow(dead_code)]
#[repr(simd)]pub struct Generic<T>([T;4]);pub fn force()->usize{std::mem::size_of::<Generic<bool>>()}
