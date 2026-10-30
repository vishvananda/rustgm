#![feature(allow_internal_unstable)]
#[allow_internal_unstable(rustc_attrs)]
macro_rules! owner{()=>{#[rustc_has_incoherent_inherent_impls]pub trait T{}}}
owner!();
