#![feature(rustc_attrs)]
fn runtime()->u32{7}pub struct S;
impl S{#[rustc_should_not_be_called_on_const_items]pub const fn f()->u32{runtime()}}
