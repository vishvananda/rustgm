#![feature(rustc_attrs)]
#[rustc_no_writable]unsafe fn f(x:u32)->u32{x}pub fn g()->u32{f(7)}
