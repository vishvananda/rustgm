#![feature(rustc_attrs)]
#[rustc_no_writable]const fn f(x:u32)->u32{x}pub const V:u32=f(7);
