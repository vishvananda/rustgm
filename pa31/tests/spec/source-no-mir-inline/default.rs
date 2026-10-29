#![feature(rustc_attrs)]
pub trait A{#[rustc_no_mir_inline]fn f(&self)->u32{7}}
