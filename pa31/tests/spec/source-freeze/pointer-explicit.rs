#![feature(no_core,lang_items,rustc_attrs,const_trait_impl,auto_traits,negative_impls,freeze_impls)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
struct N;impl !Freeze for N{}unsafe impl<T:PointeeSized>Freeze for *const T{}fn need<T:Freeze>()->u8{7}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{need::<*const N>()}