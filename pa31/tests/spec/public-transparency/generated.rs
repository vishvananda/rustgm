#![feature(rustc_attrs)]
macro_rules! make{()=>{#[repr(transparent)]#[rustc_pub_transparent]pub struct Value{inner:u64}}}make!();
pub fn observed()->u64{Value{inner:7}.inner}
