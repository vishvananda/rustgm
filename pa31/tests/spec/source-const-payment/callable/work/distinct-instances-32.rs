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
const fn need<F:const FnOnce<()>>(f:&F)->u8{7}const fn item<const K:usize>()->u8{1}#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{need(&item::<0>);need(&item::<1>);need(&item::<2>);need(&item::<3>);need(&item::<4>);need(&item::<5>);need(&item::<6>);need(&item::<7>);need(&item::<8>);need(&item::<9>);need(&item::<10>);need(&item::<11>);need(&item::<12>);need(&item::<13>);need(&item::<14>);need(&item::<15>);need(&item::<16>);need(&item::<17>);need(&item::<18>);need(&item::<19>);need(&item::<20>);need(&item::<21>);need(&item::<22>);need(&item::<23>);need(&item::<24>);need(&item::<25>);need(&item::<26>);need(&item::<27>);need(&item::<28>);need(&item::<29>);need(&item::<30>);need(&item::<31>)}
