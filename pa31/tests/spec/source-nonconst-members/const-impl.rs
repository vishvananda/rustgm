#![feature(rustc_attrs,const_trait_impl)]
#![allow(internal_features,dead_code)]
const trait Mixed{#[rustc_non_const_trait_method]fn runtime(&self)->u8;fn constant(&self)->u8;}const impl Mixed for(){fn runtime(&self)->u8{7}fn constant(&self)->u8{13}}const VALUE:u8=().constant();fn check()->u8{().runtime()}
