#![feature(rustc_attrs,const_trait_impl)]
#![allow(internal_features,dead_code)]
fn helper()->u8{7}const trait Mixed{#[rustc_non_const_trait_method]fn runtime(&self)->u8;}const impl Mixed for(){fn runtime(&self)->u8{helper()}}
