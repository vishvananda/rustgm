#![feature(rustc_attrs,repr_simd)]
#[repr(simd,packed)]#[rustc_simd_monomorphize_lane_limit="8"]struct V<T,const N:usize>([T;N]);#[unsafe(no_mangle)]pub extern "C" fn native_observed()->u64{std::mem::size_of::<V<i32,6>>() as u64*100+std::mem::align_of::<V<i32,6>>() as u64}
