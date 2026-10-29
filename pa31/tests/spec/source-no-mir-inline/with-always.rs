#![feature(rustc_attrs)]
#[rustc_no_mir_inline]#[inline(always)]pub fn f(x:u32)->u32{x}
