#![feature(rustc_attrs,repr_simd)]
#[rustc_simd_monomorphize_lane_limit=""]struct S(u32);
