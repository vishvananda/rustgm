// Independently authored DispatchFromDyn reducer; not the full Formatter provider.
#![feature(no_core,lang_items,rustc_attrs,unsize,coerce_unsized,dispatch_from_dyn)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
#[lang="unsize"]pub trait Unsize<T:PointeeSized>:PointeeSized{}
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized{}
#[lang="dispatch_from_dyn"]pub trait DispatchFromDyn<T>:Sized{}
impl<T:PointeeSized,U:PointeeSized> DispatchFromDyn<*mut U> for *const T{}
