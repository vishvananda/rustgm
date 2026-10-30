#![feature(rustc_attrs)]
trait T{#[rustc_should_not_be_called_on_const_items]fn f(&self)->u32;}
