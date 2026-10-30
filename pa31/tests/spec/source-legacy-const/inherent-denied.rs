#![feature(rustc_attrs)]
pub struct S;impl S{#[rustc_legacy_const_generics(0)]pub fn f<const N:usize>()->usize{N}}
