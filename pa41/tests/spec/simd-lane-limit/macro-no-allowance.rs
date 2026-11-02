macro_rules! make{($m:meta)=>{#[$m]struct S(u32);}}make!(rustc_simd_monomorphize_lane_limit="8");
