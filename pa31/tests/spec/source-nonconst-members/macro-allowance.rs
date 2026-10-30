#![feature(const_trait_impl,allow_internal_unstable)]
#![allow(internal_features,dead_code)]
fn helper()->u8{7}#[allow_internal_unstable(rustc_attrs)]macro_rules! member{()=>{#[rustc_non_const_trait_method]fn f()->u8{helper()}}}const trait Mixed{member!();}
