#![feature(unboxed_closures)]
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
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[lang="tuple_trait"]#[rustc_deny_explicit_impl]pub trait Tuple{}
#[lang="fn_once"]pub const trait FnOnce<Args:Tuple>{#[lang="fn_once_output"]type Output;extern "rust-call" fn call_once(self,args:Args)->Self::Output;}
#[lang="fn_mut"]pub const trait FnMut<Args:Tuple>:FnOnce<Args>{extern "rust-call" fn call_mut(&mut self,args:Args)->Self::Output;}
#[lang="fn"]pub const trait Fn<Args:Tuple>:[const]FnMut<Args>{extern "rust-call" fn call(&self,args:Args)->Self::Output;}
const fn unused<F:[const]FnOnce<()>>()->u8{7}#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>();unused::<fn()->u8>()}
