#![feature(rustc_attrs,repr_simd)]
struct S{#[rustc_simd_monomorphize_lane_limit="8"]x:u32}
