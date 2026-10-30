#![feature(rustc_attrs)]
pub struct S{v:u32}impl S{#[rustc_confusables("swap")]pub fn replace(&mut self,v:u32)->u32{let old=self.v;self.v=v;old}}pub fn observe()->u32{let mut x=S{v:7};let old=x.replace(9);old+x.v}
