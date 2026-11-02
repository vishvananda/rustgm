#![feature(rustc_attrs,repr_simd)]
#[repr(simd,packed)]#[rustc_simd_monomorphize_lane_limit="8"]struct V([i32;8]);pub fn observe()->usize{core::mem::size_of::<V>()}
