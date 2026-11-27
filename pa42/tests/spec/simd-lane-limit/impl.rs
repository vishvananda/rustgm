#![feature(rustc_attrs,repr_simd)]
#[rustc_simd_monomorphize_lane_limit="8"]impl S{}
struct S;
