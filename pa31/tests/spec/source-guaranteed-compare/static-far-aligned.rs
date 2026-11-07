#![feature(intrinsics,rustc_attrs,register_tool)]
#![register_tool(miri)]
#![allow(internal_features,dead_code,unused_variables)]
#[rustc_intrinsic]
#[rustc_nounwind]
#[rustc_do_not_const_check]
#[inline]
#[miri::intrinsic_fallback_is_spec]
pub const fn ptr_guaranteed_cmp<T>(ptr: *const T, other: *const T) -> u8 {
    (ptr == other) as u8
}

static A:u32=17;
const ACTUAL:u8=ptr_guaranteed_cmp((&raw const A as *const u8).wrapping_add(8),0usize as *const u8);
const _: [();2]=[();ACTUAL as usize];
