#![feature(rustc_attrs)]
pub struct S;impl S{#[rustc_confusables("swap")]pub unsafe fn replace()->u32{7}}pub fn observe()->u32{S::replace()}
