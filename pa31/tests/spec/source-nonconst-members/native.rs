#![feature(rustc_attrs,const_trait_impl)]
#![allow(internal_features,dead_code)]
fn helper()->u8{7}const trait Mixed{#[rustc_non_const_trait_method]fn runtime(&self)->u8{helper()}#[rustc_non_const_trait_method]fn required(&self)->u8;fn constant(&self)->u8{13}}const impl Mixed for(){fn required(&self)->u8{11}}const VALUE:u8=().constant();fn main(){println!("{} {} {}",().runtime(),().required(),VALUE)}
