#![feature(rustc_attrs,repr_simd)]
#[rustc_simd_monomorphize_lane_limit="1_000"]struct S(u32);
