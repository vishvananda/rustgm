#![feature(rustc_attrs,const_trait_impl)]
#![allow(internal_features,dead_code)]
const trait Mixed{#[unsafe(rustc_non_const_trait_method)]fn f();}
