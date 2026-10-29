#![feature(allow_internal_unstable)]
#[allow_internal_unstable(rustc_attrs)]macro_rules! make{()=>{#[rustc_no_mir_inline]pub fn f()->u32{7}}}make!();
