#![feature(no_core,lang_items,fundamental)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}

#[fundamental]pub struct Wrap{pub value:u8}#[fundamental]trait Bound{fn value()->u8;}impl Bound for Wrap{fn value()->u8{7}}#[unsafe(no_mangle)]pub extern "C" fn selected()->u8{<Wrap as Bound>::value()}#[unsafe(no_mangle)]pub extern "C" fn ordinary()->u8{Wrap{value:9}.value}
fn main(){}
