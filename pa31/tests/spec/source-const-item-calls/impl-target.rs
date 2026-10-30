#![feature(rustc_attrs)]
pub struct S;
#[rustc_should_not_be_called_on_const_items]impl S{}
