#![feature(no_core,lang_items,rustc_attrs,min_specialization)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[rustc_specialization_trait]#[rustc_deny_explicit_impl]#[lang="pointee_sized"]pub trait PointeeSized{}
#[rustc_specialization_trait]#[rustc_deny_explicit_impl]#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[rustc_specialization_trait]#[rustc_deny_explicit_impl]#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}

#[rustc_specialization_trait]trait Always{}impl<T> Always for T{}
fn main(){}
