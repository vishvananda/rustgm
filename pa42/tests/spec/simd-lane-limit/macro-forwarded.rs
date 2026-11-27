#![feature(allow_internal_unstable)]
#[allow_internal_unstable(rustc_attrs)]macro_rules! make{($i:item)=>{$i}}make!(#[rustc_simd_monomorphize_lane_limit="8"]struct S;);
