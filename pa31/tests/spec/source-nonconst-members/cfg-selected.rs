#![feature(rustc_attrs,const_trait_impl)]
#![allow(internal_features,dead_code)]
fn helper()->u8{7}const trait Mixed{#[cfg_attr(all(),rustc_non_const_trait_method)]fn f()->u8{helper()}}
