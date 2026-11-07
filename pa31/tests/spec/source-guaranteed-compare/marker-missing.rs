#![feature(intrinsics,rustc_attrs,register_tool)]
#![register_tool(miri)]
#![allow(internal_features,dead_code,unused_variables)]
#[rustc_intrinsic]
#[rustc_nounwind]
#[inline]
#[miri::intrinsic_fallback_is_spec]
pub const fn ptr_guaranteed_cmp<T>(ptr: *const T, other: *const T) -> u8 {
    (ptr == other) as u8
}
