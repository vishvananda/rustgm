#![feature(rustc_attrs)]
pub struct S;
trait T{fn f(&self)->u32;}impl T for S{#[rustc_should_not_be_called_on_const_items]fn f(&self)->u32{7}}pub fn observe()->u32{<S as T>::f(&S)}
