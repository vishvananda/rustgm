#![feature(rustc_attrs,const_trait_impl)]
#![allow(internal_features,dead_code)]
const trait Mixed{#[cfg_attr(any(),rustc_non_const_trait_method="bad")]fn f()->u8{7}}
