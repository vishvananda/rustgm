#![feature(prelude_import)]
#![feature(const_trait_impl,const_destruct)]
#![feature(auto_traits)]
#![feature(intrinsics)]
#![feature(unboxed_closures)]
#![feature(no_core,lang_items,rustc_attrs,staged_api)]
#![no_core]
#![unstable(feature="receiver_fixture",issue="none")]
#![allow(internal_features,dead_code,unused_variables)]
pub mod prelude{pub use crate::Sized;}
#[prelude_import]use crate::prelude::*;
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u32{fn clone(&self)->Self{*self}}impl Copy for u32{}
impl<T:PointeeSized> Clone for *const T{fn clone(&self)->Self{*self}}impl<T:PointeeSized> Copy for *const T{}
impl<T:PointeeSized> Clone for *mut T{fn clone(&self)->Self{*self}}impl<T:PointeeSized> Copy for *mut T{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[rustc_intrinsic]pub const fn wrapping_add<T:Copy>(a:T,b:T)->T;
#[rustc_const_unstable(feature="const_destruct",issue="133214")]#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]#[rustc_const_unstable(feature="const_destruct",issue="none")]pub const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
pub mod marker{#[lang="tuple_trait"]pub trait Tuple{}}
pub mod future{#[lang="future_trait"]pub trait Future{#[lang="future_output"]type Output;}}
pub trait Alternate<T>{#[lang="async_fn_once_output"]type Different;}
