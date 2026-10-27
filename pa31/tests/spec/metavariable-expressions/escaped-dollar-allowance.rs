#![feature(no_core,lang_items,allow_internal_unstable)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[allow_internal_unstable(macro_metavar_expr)]macro_rules! outer{()=>{$$}}
