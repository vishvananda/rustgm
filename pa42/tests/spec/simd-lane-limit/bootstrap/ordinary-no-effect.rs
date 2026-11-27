#![feature(rustc_attrs,repr_simd)]
#[rustc_simd_monomorphize_lane_limit="0"]struct V([i32;9]);pub fn observe()->usize{std::mem::size_of::<V>()}
