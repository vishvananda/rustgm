#![feature(intrinsics,rustc_attrs,register_tool)]
#![register_tool(miri)]
#![allow(internal_features,dead_code,unused_variables)]
pub fn ptr_guaranteed_cmp<T>(ptr:*const T,other:*const T)->u8{99}
const VALUE:u8=ptr_guaranteed_cmp(0usize as *const u8,0usize as *const u8);