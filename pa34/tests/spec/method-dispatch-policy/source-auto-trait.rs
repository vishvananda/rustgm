#![feature(no_core,lang_items,rustc_attrs,auto_traits)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[rustc_diagnostic_item="Send"]#[rustc_skip_during_method_dispatch(array,boxed_slice)]pub unsafe auto trait Send{}
