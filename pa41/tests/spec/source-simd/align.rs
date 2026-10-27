#![feature(repr_simd)]
#![allow(dead_code)]
#[repr(simd,align(32))]pub struct Aligned([f32;4]);#[unsafe(no_mangle)]pub extern "C"fn aligned()->u64{std::mem::size_of::<Aligned>() as u64|((std::mem::align_of::<Aligned>() as u64)<<8)}
