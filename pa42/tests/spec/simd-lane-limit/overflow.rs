#![feature(rustc_attrs,repr_simd)]
#[rustc_simd_monomorphize_lane_limit="18446744073709551616"]struct S(u32);
