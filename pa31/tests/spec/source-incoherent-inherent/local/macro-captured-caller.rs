#![feature(allow_internal_unstable)]
#[allow_internal_unstable(rustc_attrs)]
macro_rules! forward{($item:item)=>{$item}}
forward!{#[rustc_has_incoherent_inherent_impls]pub trait T{}}
