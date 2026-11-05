#![feature(no_core,lang_items)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables,async_fn_in_trait)]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized:PointeeSized {}
#[lang="sized"] pub trait Sized:MetaSized {}
#[lang="legacy_receiver"] pub trait LegacyReceiver:PointeeSized {}
impl<T:PointeeSized> LegacyReceiver for &T {}
impl<T:PointeeSized> LegacyReceiver for &mut T {}
#[lang="copy"] pub trait Copy:Clone {}
#[lang="clone"] pub trait Clone:Sized { #[lang="clone_fn"] fn clone(&self)->Self; }
#[lang="future_trait"] pub trait Future { #[lang="future_output"] type Output; }
#[lang="async_drop_in_place"]
pub async unsafe fn cleanup<T:?Sized>(_slot:&mut T){}
fn require<F:Future<Output=()>>(_:F){}
fn consumer<T:?Sized>(slot:&mut T){require(unsafe{cleanup(slot)});}
