#![feature(no_core,lang_items,intrinsics,rustc_attrs)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="va_list"]pub struct VaList<'a>{value:&'a u8}
#[rustc_intrinsic]
pub unsafe fn va_arg<T0>(a0:&mut VaList<'_>)->T0;
fn main(){}
