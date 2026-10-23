#![feature(no_core,lang_items,rustc_attrs,const_trait_impl,fundamental,auto_traits)]#![no_core]
#[lang="pointee_sized"]#[fundamental]#[rustc_specialization_trait]#[rustc_deny_explicit_impl]#[rustc_coinductive]pub trait PointeeSized{}
#[lang="meta_sized"]#[fundamental]#[rustc_specialization_trait]#[rustc_deny_explicit_impl]#[rustc_coinductive]pub trait MetaSized:PointeeSized{}
#[lang="sized"]#[fundamental]#[rustc_specialization_trait]#[rustc_deny_explicit_impl]#[rustc_coinductive]#[rustc_dyn_incompatible_trait]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;fn additional(&self)->u8;}
#[lang="copy"]pub trait Copy:Clone{}
