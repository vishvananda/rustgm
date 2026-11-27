#![feature(rustc_attrs,repr_simd)]
#[repr(simd,packed)]#[rustc_simd_monomorphize_lane_limit="0"]struct V<T,const N:usize>([T;N]);
