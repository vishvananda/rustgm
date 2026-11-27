#![feature(rustc_attrs,repr_simd)]
#[cfg(any())]#[rustc_simd_monomorphize_lane_limit="bad"]fn f(){}
