#![feature(rustc_attrs,const_trait_impl)]
#![allow(internal_features,dead_code)]
fn helper()->u8{7}trait Mixed{#[rustc_non_const_trait_method]fn runtime(&self)->u8{helper()}}impl Mixed for(){}fn check()->u8{().runtime()}
