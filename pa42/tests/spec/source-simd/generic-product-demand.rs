#![feature(repr_simd)]
#![allow(internal_features)]
#[repr(simd)] pub struct Generic<T>([T;4]);
pub struct Lane(u8,u8);
#[unsafe(no_mangle)] pub extern "C" fn product_lane_layout()->usize { std::mem::size_of::<Generic<Lane>>() }
