#![feature(rustc_attrs)]
pub struct S{v:u32}impl S{#[rustc_confusables("swap")]pub fn replace(&self,v:u32){self.v=v;}}
