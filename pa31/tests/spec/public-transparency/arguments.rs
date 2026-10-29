#![feature(rustc_attrs)]
#[repr(transparent)]#[rustc_pub_transparent(bogus)]pub struct Value{inner:u64}
