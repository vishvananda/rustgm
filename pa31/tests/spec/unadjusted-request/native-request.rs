#![feature(abi_unadjusted)]
#![allow(internal_features,unused_features,dead_code,unexpected_cfgs)]
#![feature(no_core,lang_items)]
#![no_core]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[unsafe(no_mangle)]pub extern "C" fn observed()->u8{7}
unsafe extern "C"{fn report_value(value:u8);}
fn main(){unsafe{report_value(observed());}}
