#![feature(rustc_attrs,repr_simd)]
#[rustc_simd_monomorphize_lane_limit=r#"8"#]struct S(u32);
