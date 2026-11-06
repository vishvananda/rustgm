#![feature(no_core,lang_items,rustc_attrs,staged_api,auto_traits,const_trait_impl,negative_impls)]
#![no_core]
#![unstable(feature="custom_auto_fixture",issue="none")]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]pub const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl<T:PointeeSized>Clone for &T{fn clone(&self)->Self{*self}}impl<T:PointeeSized>Copy for &T{}
pub mod marker{#[lang="phantom_data"]pub struct PhantomData<T:super::PointeeSized>;}
use marker::PhantomData;
impl<T:PointeeSized>Clone for PhantomData<T>{fn clone(&self)->Self{PhantomData}}
impl<T:PointeeSized>Copy for PhantomData<T>{}
pub auto trait Auto{}
fn need<T:Auto>(){}
struct Record{byte:u8,pair:(u8,[u8;2])}enum Choice{Left(u8),Right{value:u8}}union Either{left:u8,right:u8}struct Logical<T>{marker:PhantomData<T>}
pub fn check(){need::<Record>();need::<Choice>();need::<Either>();need::<(u8,[u8;2])>();need::<&u8>();need::<&mut u8>();need::<*const u8>();need::<*mut u8>();need::<Logical<u8>>();need::<fn()->u8>();}
