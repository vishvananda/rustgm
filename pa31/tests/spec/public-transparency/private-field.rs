#![feature(rustc_attrs)]
mod hidden{#[repr(transparent)]#[rustc_pub_transparent]pub struct Value{inner:u64}}
pub fn bad(value:hidden::Value)->u64{value.inner}
