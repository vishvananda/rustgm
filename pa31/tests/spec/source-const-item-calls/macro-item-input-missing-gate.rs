#![feature(allow_internal_unstable)]
pub struct S;#[allow_internal_unstable(rustc_attrs)]macro_rules! make{($i:item)=>{$i}}make!(impl S{#[rustc_should_not_be_called_on_const_items]pub fn f()->u32{7}});
