#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#![feature(auto_traits)]
#[rustc_has_incoherent_inherent_impls]
auto trait Auto{}
impl dyn Auto{#[rustc_allow_incoherent_impl]fn value(&self)->u8{7}}
