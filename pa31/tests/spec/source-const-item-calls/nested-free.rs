#![feature(rustc_attrs)]
pub struct S;
impl S{pub fn parent(){#[rustc_should_not_be_called_on_const_items]fn f()->u32{7}let _=f();}}
