#![feature(no_core,lang_items)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="Poll"]pub enum Actual<T>{#[lang="Ready"]Ready(T),#[lang="Pending"]Pending}
#[lang="Context"]pub struct ActualContext<'a>{token:&'a ()}
pub fn observed()->u32{match Actual::Ready(42){Actual::Ready(value)=>value,Actual::Pending=>0}}
