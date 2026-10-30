#![feature(rustc_attrs)]
#[rustc_no_writable]pub fn f(x:&mut u32)->u32{*x=7;*x}
