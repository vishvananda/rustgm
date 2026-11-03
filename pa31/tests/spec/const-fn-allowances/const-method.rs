#![feature(no_core,lang_items,rustc_attrs,staged_api)]
#![no_core]
#![stable(feature="root",since="1.0.0")]
#![allow(internal_features,dead_code,unused_variables)]
#[stable(feature="root",since="1.0.0")]#[lang="pointee_sized"]pub trait PointeeSized{}
#[stable(feature="root",since="1.0.0")]#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[stable(feature="root",since="1.0.0")]#[lang="sized"]pub trait Sized:MetaSized{}
#[stable(feature="root",since="1.0.0")]pub enum Item{}impl Item{#[rustc_allow_const_fn_unstable(door)]
#[stable(feature="root",since="1.0.0")]#[rustc_const_stable(feature="method_const",since="1.0.0")]pub const fn allowed()->u8{7}}

#[stable(feature="root",since="1.0.0")]#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
#[stable(feature="root",since="1.0.0")]impl<T:PointeeSized> LegacyReceiver for &T{}
#[stable(feature="root",since="1.0.0")]#[lang="clone"]pub trait Clone:Sized{#[stable(feature="root",since="1.0.0")]#[lang="clone_fn"]fn clone(&self)->Self;}
#[stable(feature="root",since="1.0.0")]#[lang="copy"]pub trait Copy:Clone{}
#[stable(feature="root",since="1.0.0")]impl Clone for u8{fn clone(&self)->Self{*self}}#[stable(feature="root",since="1.0.0")]impl Copy for u8{}
