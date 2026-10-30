#![feature(rustc_attrs,const_trait_impl)]
#![allow(internal_features,dead_code)]
trait Mixed{fn f();}impl Mixed for(){#[rustc_non_const_trait_method]fn f(){}}
