#![feature(rustc_attrs)]
#![deny(unused_attributes)]
pub struct S;
impl S{#[rustc_should_not_be_called_on_const_items]fn f(){}}
