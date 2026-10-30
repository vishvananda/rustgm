#![feature(rustc_attrs)]
pub struct S;
impl S{#[rustc_should_not_be_called_on_const_items]pub fn f(&self)->u32{7}}pub fn observe()->u32{S.f()}
