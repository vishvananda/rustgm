#![feature(rustc_attrs)]
pub struct S;trait T{#[rustc_confusables("swap")]fn replace(&self)->u32;}
