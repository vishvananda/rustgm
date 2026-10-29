#![feature(rustc_attrs)]
#[repr(transparent)]pub struct Value{#[rustc_pub_transparent]inner:u64}
