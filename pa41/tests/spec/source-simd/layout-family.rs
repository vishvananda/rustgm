#![feature(repr_simd)]
#![allow(dead_code)]
#[repr(simd)]struct I([i64;2]);#[repr(simd)]struct F([f32;4]);#[repr(simd)]struct D([f64;2]);#[repr(simd)]struct Odd([f32;3]);#[repr(simd,packed)]struct Packed([f32;3]);
#[unsafe(no_mangle)]pub extern "C" fn layout_bits()->u64{use std::mem::{size_of as s,align_of as a};s::<I>() as u64|(a::<I>() as u64)<<8|(s::<F>() as u64)<<16|(a::<D>() as u64)<<24|(s::<Odd>() as u64)<<32|(a::<Odd>() as u64)<<40|(s::<Packed>() as u64)<<48|(a::<Packed>() as u64)<<56}
