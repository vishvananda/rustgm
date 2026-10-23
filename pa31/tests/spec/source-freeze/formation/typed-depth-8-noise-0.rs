#![recursion_limit="1024"]
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
#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
struct W<T>(*const T);struct N;fn need<T:Freeze>()->u8{7}
type Layer0 = N;
type Layer1 = W<Layer0>;
type Layer2 = W<Layer1>;
type Layer3 = W<Layer2>;
type Layer4 = W<Layer3>;
type Layer5 = W<Layer4>;
type Layer6 = W<Layer5>;
type Layer7 = W<Layer6>;
type Layer8 = W<Layer7>;
#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{need::<Layer8>()}
