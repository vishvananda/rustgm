#![feature(no_core,lang_items,final_associated_functions)]
#![no_core]
#![allow(dead_code,internal_features)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
pub trait Share:Clone{final fn share(&self)->Self{Clone::clone(self)}}
pub struct S;
impl Clone for S{fn clone(&self)->Self{S}}
impl Share for S{}
pub fn actual_call(value:&S)->S{Share::share(value)}
