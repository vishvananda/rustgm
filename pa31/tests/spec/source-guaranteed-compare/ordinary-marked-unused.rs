#![feature(intrinsics,rustc_attrs,register_tool)]
#![register_tool(miri)]
#![allow(internal_features,dead_code,unused_variables)]
#[rustc_do_not_const_check]pub const fn ordinary<T>(ptr:*const T,other:*const T)->u8{(ptr==other)as u8}