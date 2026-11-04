#![feature(no_core,lang_items,intrinsics,rustc_attrs,transparent_unions)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="manually_drop"]#[repr(transparent)]pub struct ManuallyDrop<T>{value:T}
#[lang="bikeshed_guaranteed_no_drop"]pub trait NoDrop{}
#[lang="maybe_uninit"]#[repr(transparent)]pub union MaybeUninit<T>{empty:(),value:ManuallyDrop<T>}
#[lang="owned_box"]pub struct Box<T>{pointer:*mut T}
#[rustc_intrinsic]
pub fn write_box_via_move<T0>(a0:Box<MaybeUninit<T0>>,a1:T0)->Box<MaybeUninit<T0>>;
fn main(){}
