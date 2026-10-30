#![feature(rustc_attrs)]
pub struct S;
impl S{#[unsafe(rustc_should_not_be_called_on_const_items)]fn f(){}}
