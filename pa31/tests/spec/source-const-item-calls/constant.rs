#![feature(rustc_attrs)]
#[rustc_should_not_be_called_on_const_items]const V:u32=7;
