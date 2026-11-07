#![feature(intrinsics,rustc_attrs,register_tool)]
#![register_tool(miri)]
#![allow(internal_features,dead_code,unused_variables)]
pub const fn lookalike<T>(ptr:*const T,other:*const T)->u8{(ptr==other)as u8}