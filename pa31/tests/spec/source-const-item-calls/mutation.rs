#![feature(rustc_attrs)]
pub struct S{v:u32}impl S{#[rustc_should_not_be_called_on_const_items]pub fn change(&mut self){self.v=9;}}pub fn observe()->u32{let mut x=S{v:7};x.change();x.v}
