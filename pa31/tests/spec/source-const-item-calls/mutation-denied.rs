#![feature(rustc_attrs)]
pub struct S{v:u32}impl S{#[rustc_should_not_be_called_on_const_items]pub fn change(&self){self.v=9;}}
