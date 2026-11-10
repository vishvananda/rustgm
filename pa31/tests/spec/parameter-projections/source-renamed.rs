// Authored source protocol; roles follow pinned core iter/traits and option.
#![feature(no_core,lang_items,rustc_attrs,const_trait_impl,auto_traits,unboxed_closures,staged_api)]
#![unstable(feature="callback_fixture",issue="none")]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[rustc_const_unstable(feature="const_clone",issue="142757")]#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
#[lang="Option"]pub enum Maybe<T>{#[lang="Some"]Present(T),#[lang="None"]Absent}
#[lang="iterator"]pub trait Walk{type Item;#[lang="next"]fn advance(&mut self)->Maybe<Self::Item>;fn seek<P>(&mut self,_predicate:P)->Maybe<usize> where P:FnMut(Self::Item)->bool{loop{}}}

#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="tuple_trait"]#[rustc_deny_explicit_impl]pub trait Tuple{}
#[rustc_const_unstable(feature="const_trait_impl",issue="none")]#[lang="fn_once"]pub const trait FnOnce<Args:Tuple>{#[lang="fn_once_output"]type Output;extern "rust-call" fn call_once(self,args:Args)->Self::Output;}
#[rustc_const_unstable(feature="const_trait_impl",issue="none")]#[lang="fn_mut"]pub const trait FnMut<Args:Tuple>:FnOnce<Args>{extern "rust-call" fn call_mut(&mut self,args:Args)->Self::Output;}
#[rustc_const_unstable(feature="const_trait_impl",issue="none")]#[lang="fn"]pub const trait Fn<Args:Tuple>:[const]FnMut<Args>{extern "rust-call" fn call(&self,args:Args)->Self::Output;}
pub struct Values<'a>{value:&'a u8}impl<'a> Walk for Values<'a>{type Item=&'a u8;fn advance(&mut self)->Maybe<Self::Item>{Maybe::Absent}}pub fn probe(v:&mut Values<'_>)->Maybe<usize>{v.seek(|&c|{let _:u8=c;true})}
