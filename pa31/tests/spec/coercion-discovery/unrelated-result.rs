#![feature(no_core,lang_items,rustc_attrs,const_trait_impl,auto_traits,const_destruct)]
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
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized{}
enum Outcome<T,E>{Value(T),Fault(E)}trait ConvertFrom<S>:Sized{type Error;fn from(x:S)->Outcome<Self,Self::Error>;}trait ConvertTo<T>{type Error;fn convert(self)->Outcome<T,Self::Error>;}impl<S,T:ConvertFrom<S>> ConvertTo<T> for S{type Error=<T as ConvertFrom<S>>::Error;fn convert(self)->Outcome<T,Self::Error>{T::from(self)}}trait Make<T>{fn make(self)->T;}impl<S,T> ConvertFrom<S> for T where S:Make<T>{type Error=u16;fn from(x:S)->Outcome<Self,Self::Error>{Outcome::Value(x.make())}}struct Alignment(u8);struct Failure;struct Byte(u8);impl Byte{fn get(self)->u8{self.0}}impl ConvertFrom<u8> for Alignment{type Error=Failure;fn from(x:u8)->Outcome<Self,Self::Error>{Outcome::Value(Alignment(x))}}impl ConvertFrom<Byte> for Alignment{type Error=Failure;fn from(x:Byte)->Outcome<Alignment,Self::Error>{x.get().convert()}}fn unproved<T:ConvertFrom<u8>>(x:T::Error)->u16{x}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{match <Alignment as ConvertFrom<Byte>>::from(Byte(7)){Outcome::Value(x)=>x.0,Outcome::Fault(_)=>0}}
