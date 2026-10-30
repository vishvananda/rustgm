#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[rustc_diagnostic_item="From"]pub trait From<T>:Sized{#[rustc_diagnostic_item="from_fn"]fn from(value:T)->Self;}
impl From<f32> for f32{fn from(value:f32)->Self{32.0}}
impl From<f64> for f32{fn from(value:f64)->Self{64.0}}
#[unsafe(no_mangle)]pub extern "C" fn observed()->u8{<f32 as From<_>>::from(1.0) as u8}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:?Sized>LegacyReceiver for &T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for f32{fn clone(&self)->Self{*self}}
impl Copy for f32{}
impl Clone for f64{fn clone(&self)->Self{*self}}
impl Copy for f64{}

#[lang="From"]pub trait Actual<T>:Sized{}
