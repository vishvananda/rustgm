#![feature(rustc_attrs,const_trait_impl)]
#![allow(internal_features,dead_code)]
fn helper()->u8{7}const trait Mixed{fn runtime(&self)->u8{helper()}}
