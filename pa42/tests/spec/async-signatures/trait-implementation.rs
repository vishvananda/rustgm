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
pub trait Async { async fn compute(&self)->u8; }
struct Thing;
impl Async for Thing { async fn compute(&self)->u8{7u8} }
fn require<F:Future<Output=u8>>(_:F){}
fn consumer<T:Async>(x:&T){require(x.compute());}
