#![feature(rustc_attrs,repr_simd)]
#[rustc_simd_monomorphize_lane_limit="-1"]struct S(u32);
