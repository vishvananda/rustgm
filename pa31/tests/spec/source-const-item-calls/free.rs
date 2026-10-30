#![feature(rustc_attrs)]
#[rustc_should_not_be_called_on_const_items]pub fn f()->u32{7}
