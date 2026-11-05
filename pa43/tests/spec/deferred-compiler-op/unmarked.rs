#![feature(no_core,lang_items)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="future_trait"]pub trait Future{#[lang="future_output"]type Output;}
pub async unsafe fn cleanup<T:?Sized>(to_drop:&mut T){}

pub fn ordinary() -> u8 { 7 }
