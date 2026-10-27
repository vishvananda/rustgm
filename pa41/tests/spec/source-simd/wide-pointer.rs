#![feature(repr_simd)]
#![allow(dead_code)]
#[repr(simd)]pub struct Wide([*const str;2]);pub fn force()->usize{std::mem::size_of::<Wide>()}
