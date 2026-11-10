#![feature(no_core,lang_items,rustc_attrs,intrinsics)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[rustc_const_stable_indirect]#[rustc_nounwind]#[rustc_intrinsic]pub const fn is_val_statically_known<T:Copy>(_arg:T)->bool{false}
const RESULT:bool=is_val_statically_known(7u8);
#[unsafe(no_mangle)]pub extern "C" fn constant()->u8{RESULT as u8}
#[unsafe(no_mangle)]pub extern "C" fn runtime()->u8{if is_val_statically_known(7u8){7}else{7}}
#[unsafe(no_mangle)]pub extern "C" fn arguments()->u8{let mut count=0u8;let _hint=is_val_statically_known({count=match count{0=>1,_=>2};7u8});count}
