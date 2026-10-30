#![feature(rustc_attrs)]
pub struct S;trait T{fn replace(&self)->u32;}impl T for S{#[rustc_confusables("swap")]fn replace(&self)->u32{7}}
