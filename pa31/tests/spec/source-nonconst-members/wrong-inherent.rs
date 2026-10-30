#![feature(rustc_attrs,const_trait_impl)]
#![allow(internal_features,dead_code)]
struct Thing;impl Thing{#[rustc_non_const_trait_method]fn wrong(&self){}}
