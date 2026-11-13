#![feature(no_core,lang_items,intrinsics,rustc_attrs,repr_simd)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables,unused_unsafe)]
// Authored reducer of portable-simd's masks.rs `first_set` (pinned core 88d9e12a):
// trait and type shapes follow core_simd's vector.rs and masks.rs; bodies are reduced.
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
pub enum Option<T>{None,Some(T)}
impl Clone for i8{fn clone(&self)->Self{*self}}impl Copy for i8{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl<T:Copy,const N:usize> Clone for [T;N]{fn clone(&self)->Self{*self}}impl<T:Copy,const N:usize> Copy for [T;N]{}
#[rustc_intrinsic]pub unsafe fn simd_cast<T,U>(x:T)->U;
#[rustc_intrinsic]pub unsafe fn simd_or<T>(x:T,y:T)->T;
#[rustc_intrinsic]pub unsafe fn simd_reduce_min<T,U>(x:T)->U;
pub unsafe fn transmute_copy<Src,Dst>(src:&Src)->Dst{loop{}}
fn id<U>(u:U)->U{u}
fn convert<A,B>(a:A)->B{loop{}}
mod vsealed{pub trait Sealed{}}
pub unsafe trait SimdElement:vsealed::Sealed+Copy{type Mask:MaskElement;}
impl vsealed::Sealed for i8{}unsafe impl SimdElement for i8{type Mask=i8;}
impl vsealed::Sealed for u8{}unsafe impl SimdElement for u8{type Mask=i8;}
pub unsafe trait SimdCast:vsealed::Sealed+SimdElement{}
unsafe impl SimdCast for i8{}unsafe impl SimdCast for u8{}
mod sealed{use super::*;pub trait Sealed{fn eq(self,other:Self)->bool;fn to_usize(self)->usize;type Unsigned:SimdElement;const TRUE:Self;}}
use sealed::Sealed;
impl Sealed for i8{fn eq(self,other:Self)->bool{loop{}}fn to_usize(self)->usize{loop{}}type Unsigned=u8;const TRUE:Self=0;}
pub unsafe trait MaskElement:SimdElement<Mask=Self>+SimdCast+Sealed{}
unsafe impl MaskElement for i8{}
#[repr(simd)]pub struct Simd<T,const N:usize>([T;N]) where T:SimdElement;
impl<T,const N:usize> Clone for Simd<T,N> where T:SimdElement{fn clone(&self)->Self{*self}}
impl<T,const N:usize> Copy for Simd<T,N> where T:SimdElement{}
pub struct Mask<T,const N:usize>(Simd<T,N>) where T:MaskElement;
impl<T,const N:usize> Mask<T,N> where T:MaskElement{pub fn to_simd(self)->Simd<T,N>{self.0}}
impl<T,const N:usize> Mask<T,N> where T:MaskElement{
    pub fn least(self)->T{
        let r:T::Unsigned=convert(self.to_simd());
        unsafe{transmute_copy(&r)}
    }
}
