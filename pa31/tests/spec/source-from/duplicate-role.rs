#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="From"]pub trait From<T>:Sized{fn from(value:T)->Self;}
#[lang="From"]pub trait Other<T>:Sized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:?Sized>LegacyReceiver for &T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for f32{fn clone(&self)->Self{*self}}
impl Copy for f32{}
impl Clone for f64{fn clone(&self)->Self{*self}}
impl Copy for f64{}
