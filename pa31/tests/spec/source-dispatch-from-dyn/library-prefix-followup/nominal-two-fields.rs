// Independently authored DispatchFromDyn reducer; not the full Formatter provider.
#![feature(no_core,lang_items,rustc_attrs,staged_api,unsize,coerce_unsized,dispatch_from_dyn)]
#![no_core]
#![unstable(feature="dispatch_fixture",issue="none")]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
#[unstable(feature="unsize",issue="18598")]
#[lang="unsize"]pub trait Unsize<T:PointeeSized>:PointeeSized{}
#[unstable(feature="coerce_unsized",issue="18598")]
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized{}
#[unstable(feature="dispatch_from_dyn",issue="none")]
#[lang="dispatch_from_dyn"]pub trait DispatchFromDyn<T>:Sized{}
impl<T:PointeeSized,U:PointeeSized> DispatchFromDyn<*const U> for *const T{}
pub struct W<T:PointeeSized>{first:*const T,second:*const T}
impl<T:PointeeSized,U:PointeeSized> DispatchFromDyn<W<U>> for W<T>{}
