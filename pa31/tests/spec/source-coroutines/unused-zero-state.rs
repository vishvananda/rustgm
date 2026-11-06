#![feature(fundamental,coroutines,stmt_expr_attributes)]
#![feature(no_core,lang_items,rustc_attrs,staged_api,auto_traits,const_trait_impl,prelude_import)]
#![no_core]
#![unstable(feature="async_iterator_fixture",issue="none")]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]pub const trait Destruct:PointeeSized{}
#[lang="unpin"]pub auto trait Unpin{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
pub enum Option<T>{None,Some(T)}
pub mod prelude{pub use crate::{Sized,Unpin,Option,Option::{Some,None}};}
#[prelude_import]use crate::prelude::*;
pub mod ops{
#[lang="deref"]pub trait Deref:PointeeSized{#[lang="deref_target"]type Target:PointeeSized;fn deref(&self)->&Self::Target;}
#[lang="deref_mut"]pub trait DerefMut:Deref{fn deref_mut(&mut self)->&mut Self::Target;}
use crate::{PointeeSized,LegacyReceiver};
impl<T:PointeeSized>Deref for &T{type Target=T;fn deref(&self)->&T{*self}}
impl<T:PointeeSized>Deref for &mut T{type Target=T;fn deref(&self)->&T{&**self}}
impl<T:PointeeSized>DerefMut for &mut T{fn deref_mut(&mut self)->&mut T{&mut **self}}
}
pub mod pin{
use crate::{ops::{Deref,DerefMut},PointeeSized,Unpin,LegacyReceiver};
#[lang="pin"]#[repr(transparent)]pub struct Pin<P>{pub pointer:P}
impl<P>Pin<P>{#[lang="new_unchecked"]pub const unsafe fn new_unchecked(pointer:P)->Self{Self{pointer}}}
impl<P:Deref>Pin<P>{pub fn new(pointer:P)->Self where P::Target:Unpin{Self{pointer}}}
impl<P:Deref>Deref for Pin<P>{type Target=P::Target;fn deref(&self)->&Self::Target{self.pointer.deref()}}
impl<P:DerefMut>DerefMut for Pin<P> where P::Target:Unpin{fn deref_mut(&mut self)->&mut Self::Target{self.pointer.deref_mut()}}
impl<P:DerefMut>Pin<P>{pub fn as_deref_mut(self:Pin<&mut Self>)->Pin<&mut P::Target>{unsafe{Pin::new_unchecked(self.pointer.pointer.deref_mut())}}}
impl<P:Deref>LegacyReceiver for Pin<P>{}
}
#[lang="coroutine_state"]pub enum State{Only}
