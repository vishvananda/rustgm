#![feature(rustc_attrs)]
pub struct S;impl S{#[rustc_should_not_be_called_on_const_items]#[rustc_no_mir_inline]#[rustc_no_writable]pub fn f()->u32{7}}
