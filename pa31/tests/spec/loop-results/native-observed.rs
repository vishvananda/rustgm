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
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
pub struct Raw<T>{ptr:*mut T}pub struct Cursor<'a,T:'a>{first:*mut T,second:*mut T,marker:&'a mut ()}impl<'a,T>Cursor<'a,T>{fn post_inc_start(&mut self)->Raw<T>{let ptr=self.first;self.first=self.second;Raw{ptr}}fn take_ptr(&mut self)->*mut T{let ptr=self.first;self.first=self.second;ptr}fn tied_next(&mut self)->&mut T{let ptr=self.first;self.first=self.second;unsafe{&mut *ptr}}}
impl<T> Raw<T>{unsafe fn as_mut<'a>(&mut self)->&'a mut T{unsafe{&mut *self.ptr}}}#[unsafe(no_mangle)]pub unsafe extern "C" fn answer(first:*mut u8,second:*mut u8)->u8{let mut initial=0u8;let mut marker=();let mut cursor=Cursor{first,second,marker:&mut marker};let mut result=&mut initial;let mut done=false;loop{result=unsafe{cursor.post_inc_start().as_mut()};if done{break}done=true;}*result=9;*result}
