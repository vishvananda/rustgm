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
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl dyn Send{pub fn value(&self)->u8{7}}
