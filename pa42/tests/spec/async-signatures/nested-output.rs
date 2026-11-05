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
impl Clone for u8 { fn clone(&self)->u8{*self} }
impl Copy for u8 {}
#[lang="future_trait"] pub trait Future { #[lang="future_output"] type Output; }
pub async fn value()->impl Copy{7u8}
fn require<F:Future>(_:F)where F::Output:Copy{}
fn consumer(){require(value());}
