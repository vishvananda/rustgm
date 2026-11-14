#![feature(no_core,lang_items,intrinsics,rustc_attrs,const_trait_impl,auto_traits,transparent_unions,decl_macro,allow_internal_unstable)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="unsize"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub trait Unsize<T:PointeeSized>:PointeeSized{}
#[lang="coerce_unsized"]pub trait CoerceUnsized<T:PointeeSized>:Sized{}
impl<'a,'b:'a,T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<&'a mut U> for &'b mut T{}
impl<T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<*mut U> for *mut T{}
#[lang="sync"]pub unsafe auto trait Sync{}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
impl<T> Clone for *mut T{fn clone(&self)->Self{*self}}impl<T> Copy for *mut T{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
pub struct Raw<T>{ptr:*mut T}pub struct Cursor<'a,T:'a>{first:*mut T,second:*mut T,marker:&'a mut ()}impl<'a,T>Cursor<'a,T>{fn post_inc_start(&mut self)->Raw<T>{let ptr=self.first;self.first=self.second;Raw{ptr}}fn take_ptr(&mut self)->*mut T{let ptr=self.first;self.first=self.second;ptr}fn tied_next(&mut self)->&mut T{let ptr=self.first;self.first=self.second;unsafe{&mut *ptr}}}
impl<T> Raw<T>{unsafe fn as_mut<'a>(&mut self)->&'a mut T{unsafe{&mut *self.ptr}}}pub unsafe fn run<'a,T>(cursor:&mut Cursor<'a,T>,slot:&mut &'a mut T,stop:bool){loop{*slot=unsafe{cursor.post_inc_start().as_mut()};if stop{break}}}
