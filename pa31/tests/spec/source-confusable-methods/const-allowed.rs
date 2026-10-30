#![feature(rustc_attrs)]
pub struct S;impl S{#[rustc_confusables("swap")]pub const fn replace(&self)->u32{7}}pub const V:u32=S.replace();
