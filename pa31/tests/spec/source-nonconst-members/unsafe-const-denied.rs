#![feature(rustc_attrs,const_trait_impl)]
#![allow(internal_features,dead_code)]
const trait Mixed{#[rustc_non_const_trait_method]unsafe fn f()->u8{7}}const impl Mixed for(){}const V:u8=unsafe{<() as Mixed>::f()};
