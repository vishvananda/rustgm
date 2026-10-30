#![feature(rustc_attrs)]
pub struct S;impl S{#[rustc_confusables("not a method name")]pub fn replace(&self)->u32{7}}pub fn observe()->u32{S.replace()}
