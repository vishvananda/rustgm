#![feature(rustc_attrs)]
pub trait A{#[rustc_no_writable]fn f(&self)->u32{7}}
