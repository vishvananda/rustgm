#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
const impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
const fn unused<T:[const]Clone>()->u8{7}const fn caller_0<T:[const]Clone>()->u8{unused::<T>()}const fn caller_1<T:[const]Clone>()->u8{unused::<T>()}const fn caller_2<T:[const]Clone>()->u8{unused::<T>()}const fn caller_3<T:[const]Clone>()->u8{unused::<T>()}const fn caller_4<T:[const]Clone>()->u8{unused::<T>()}const fn caller_5<T:[const]Clone>()->u8{unused::<T>()}const fn caller_6<T:[const]Clone>()->u8{unused::<T>()}const fn caller_7<T:[const]Clone>()->u8{unused::<T>()}const N:u8={caller_0::<u8>();caller_1::<u8>();caller_2::<u8>();caller_3::<u8>();caller_4::<u8>();caller_5::<u8>();caller_6::<u8>();caller_7::<u8>()};#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{N}
