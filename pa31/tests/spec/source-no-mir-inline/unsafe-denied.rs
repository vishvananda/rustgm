#![feature(rustc_attrs)]
#[rustc_no_mir_inline]unsafe fn f(x:u32)->u32{x}pub fn g()->u32{f(7)}
