#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
macro_rules! owner{()=>{#[rustc_has_incoherent_inherent_impls]pub trait T{}}}
owner!();
