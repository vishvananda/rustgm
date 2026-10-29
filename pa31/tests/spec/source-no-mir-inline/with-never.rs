#![feature(rustc_attrs)]
#[rustc_no_mir_inline]#[inline(never)]pub fn f(x:u32)->u32{x}
