#![feature(intrinsics,rustc_attrs,register_tool)]
#![register_tool(miri)]
#![allow(internal_features,dead_code,unused_variables)]
fn ordinary()->u8{99}
#[rustc_intrinsic]
#[rustc_nounwind]
#[rustc_do_not_const_check]
#[inline]
#[miri::intrinsic_fallback_is_spec]
pub const fn ptr_guaranteed_cmp<T>(ptr: *const T, other: *const T) -> u8 {
    ordinary()
}

const CHECK:u8=ptr_guaranteed_cmp(0usize as *const u8,0usize as *const u8);
#[unsafe(no_mangle)]pub extern "C" fn observed(a:*const u8,b:*const u8)->u8{ptr_guaranteed_cmp(a,b)}
#[unsafe(no_mangle)]pub extern "C" fn observed_const()->u8{CHECK}
