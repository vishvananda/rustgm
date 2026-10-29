#![feature(rustc_attrs)]
pub trait A{fn f(&self)->u32;}impl A for u32{#[rustc_no_mir_inline]fn f(&self)->u32{*self}}
