#![feature(no_core,lang_items,rustc_attrs,auto_traits,negative_impls)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[rustc_diagnostic_item="Send"]pub unsafe auto trait Send{}
#[lang="sync"]#[rustc_diagnostic_item="Sync"]pub unsafe auto trait Sync{}
#[lang="unpin"]pub auto trait Unpin{}
pub trait Any{}
fn need<T:Send+?Sized>(){}fn share<T:Sync+?Sized>(){}fn check(){need::<dyn Any+Send>();need::<dyn Any+Send+Sync>();share::<dyn Any+Send+Sync>();}
