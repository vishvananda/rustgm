#![feature(rustc_attrs,repr_simd)]
#[unsafe(rustc_simd_monomorphize_lane_limit="8")]struct S(u32);
