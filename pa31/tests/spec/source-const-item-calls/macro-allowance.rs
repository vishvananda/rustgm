#![feature(allow_internal_unstable)]
pub struct S;
#[allow_internal_unstable(rustc_attrs)]macro_rules! make{()=>{impl S{#[rustc_should_not_be_called_on_const_items]pub fn f()->u32{7}}}}make!();pub fn observe()->u32{S::f()}
