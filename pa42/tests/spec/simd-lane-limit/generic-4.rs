#![feature(rustc_attrs,repr_simd)]
#[repr(simd,packed)]#[rustc_simd_monomorphize_lane_limit="8"]struct V<T,const N:usize>([T;N]);pub fn observe()->usize{core::mem::size_of::<V<i32,4>>()}
