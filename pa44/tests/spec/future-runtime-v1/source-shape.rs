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
impl Clone for u8 { fn clone(&self)->Self{*self} }
impl Copy for u8 {}
pub async fn scalar(x:u8)->u8{x}
pub async fn raw(_: *mut u8)->u8{7}
pub async fn output()->*mut u8{0usize as *mut u8}
pub async fn generic<T>(x:T)->T{x}
pub async fn borrowed<'a>(x:&'a mut u8)->u8{*x}
