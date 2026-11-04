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
#[rustc_intrinsic]
pub fn offload<T0,T1,T2>(a0:T0,a1:[u32;3],a2:[u32;3],a3:u32,a4:T1,extra:u8)->T2;
fn main(){}
