#![feature(repr_simd)]
#![allow(internal_features)]
#[repr(simd)] pub struct Generic<T>([T;4]);
pub struct Lane(u8);
#[unsafe(no_mangle)] pub extern "C" fn scalar_lane_layout()->usize { std::mem::size_of::<Generic<Lane>>() }
#[unsafe(no_mangle)] pub extern "C" fn bool_lane_layout()->usize { std::mem::size_of::<Generic<bool>>() }
