#![feature(rustc_attrs,const_trait_impl)]
#![allow(internal_features,dead_code)]
fn helper()->u8{7}macro_rules! member{()=>{#[rustc_non_const_trait_method]fn f()->u8{helper()}}}const trait Mixed{member!();}
