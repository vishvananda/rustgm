#![feature(rustc_attrs)]
#[rustc_no_writable]fn f(x:u32)->u32{x}pub const V:u32=f(7);
