#![feature(no_core,lang_items,intrinsics,rustc_attrs,repr_simd,const_trait_impl,auto_traits)]
#![no_core]
#![rustc_coherence_is_core]
#![allow(internal_features,dead_code,unused_variables,unused_unsafe,incomplete_features)]
// Authored reducer of portable-simd's ops.rs `int_divrem_guard` Div for Simd<i8, N>
// (pinned core 88d9e12a): trait and type shapes follow core_simd's vector.rs, masks.rs,
// select.rs, simd/cmp/eq.rs and ops.rs; bodies are reduced.
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="freeze"] pub unsafe auto trait Freeze {}
#[lang="not"]pub const trait Not{type Output;fn not(self)->Self::Output;}
const impl Not for bool{type Output=bool;fn not(self)->bool{!self}}
#[lang="eq"]pub const trait PartialEq<Rhs:PointeeSized=Self>:PointeeSized{fn eq(&self,other:&Rhs)->bool;fn ne(&self,other:&Rhs)->bool{!self.eq(other)}}
#[lang="neg"]pub trait Neg{type Output;fn neg(self)->Self::Output;}
#[lang="bitand"]pub trait BitAnd<Rhs=Self>{type Output;fn bitand(self,rhs:Rhs)->Self::Output;}
#[lang="div"]pub trait Div<Rhs=Self>{type Output;fn div(self,rhs:Rhs)->Self::Output;}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
impl Clone for i8{fn clone(&self)->Self{*self}}impl Copy for i8{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for i32{fn clone(&self)->Self{*self}}impl Copy for i32{}
impl Clone for u16{fn clone(&self)->Self{*self}}impl Copy for u16{}
const impl PartialEq for i8{fn eq(&self,other:&Self)->bool{*self==*other}fn ne(&self,other:&Self)->bool{*self!=*other}}
const impl PartialEq for u8{fn eq(&self,other:&Self)->bool{*self==*other}fn ne(&self,other:&Self)->bool{*self!=*other}}
impl Neg for i8{type Output=i8;fn neg(self)->i8{loop{}}}
impl Neg for i32{type Output=i32;fn neg(self)->i32{loop{}}}
impl Neg for i64{type Output=i64;fn neg(self)->i64{loop{}}}
impl i8{pub const MIN:i8=-128;}
impl u8{pub const MIN:u8=0;}
impl<T:Copy,const N:usize> Clone for [T;N]{fn clone(&self)->Self{*self}}impl<T:Copy,const N:usize> Copy for [T;N]{}
#[rustc_intrinsic]pub unsafe fn simd_div<T>(x:T,y:T)->T;
#[rustc_intrinsic]pub unsafe fn simd_and<T>(x:T,y:T)->T;
fn panic_zero(message:&'static str)->!{loop{}}
mod vsealed{pub trait Sealed{}}
pub unsafe trait SimdElement:vsealed::Sealed+Copy{type Mask:MaskElement;}
impl vsealed::Sealed for i8{}unsafe impl SimdElement for i8{type Mask=i8;}
impl vsealed::Sealed for u8{}unsafe impl SimdElement for u8{type Mask=i8;}
impl vsealed::Sealed for i32{}unsafe impl SimdElement for i32{type Mask=i32;}
impl vsealed::Sealed for u16{}unsafe impl SimdElement for u16{type Mask=i8;}
mod sealed{pub trait Sealed{const TRUE:Self;const FALSE:Self;}}
use sealed::Sealed;
impl Sealed for i8{const TRUE:Self=-1;const FALSE:Self=0;}
impl Sealed for i32{const TRUE:Self=-1;const FALSE:Self=0;}
pub unsafe trait MaskElement:SimdElement<Mask=Self>+Sealed{}
unsafe impl MaskElement for i8{}
unsafe impl MaskElement for i32{}
#[repr(simd)]pub struct Simd<T,const N:usize>([T;N]) where T:SimdElement;
impl<T,const N:usize> Clone for Simd<T,N> where T:SimdElement{fn clone(&self)->Self{*self}}
impl<T,const N:usize> Copy for Simd<T,N> where T:SimdElement{}
impl<T,const N:usize> Simd<T,N> where T:SimdElement{pub const fn splat(value:T)->Self{loop{}}}
#[repr(transparent)]pub struct Mask<T,const N:usize>(Simd<T,N>) where T:MaskElement;
impl<T,const N:usize> Clone for Mask<T,N> where T:MaskElement{fn clone(&self)->Self{*self}}
impl<T,const N:usize> Copy for Mask<T,N> where T:MaskElement{}
impl<T,const N:usize> Mask<T,N> where T:MaskElement{
    pub const fn splat(value:bool)->Self{Self(Simd::splat(if value{T::TRUE}else{T::FALSE}))}
    pub fn to_simd(self)->Simd<T,N>{self.0}
    pub unsafe fn from_simd_unchecked(value:Simd<T,N>)->Self{Self(value)}
    pub fn any(self)->bool{loop{}}
}
impl<T,const N:usize> BitAnd for Mask<T,N> where T:MaskElement{type Output=Self;fn bitand(self,rhs:Self)->Self{unsafe{Self(simd_and(self.0,rhs.0))}}}
impl<T,const N:usize> BitAnd<bool> for Mask<T,N> where T:MaskElement{type Output=Self;fn bitand(self,rhs:bool)->Self{self&Self::splat(rhs)}}
impl<T,const N:usize> BitAnd<Mask<T,N>> for bool where T:MaskElement{type Output=Mask<T,N>;fn bitand(self,rhs:Mask<T,N>)->Mask<T,N>{Mask::splat(self)&rhs}}
pub trait SimdPartialEq{type Mask;fn simd_eq(self,other:Self)->Self::Mask;}
impl<const N:usize> SimdPartialEq for Simd<i8,N>{type Mask=Mask<<i8 as SimdElement>::Mask,N>;fn simd_eq(self,other:Self)->Self::Mask{loop{}}}
impl<const N:usize> SimdPartialEq for Simd<u8,N>{type Mask=Mask<<u8 as SimdElement>::Mask,N>;fn simd_eq(self,other:Self)->Self::Mask{loop{}}}
impl<const N:usize> SimdPartialEq for Mask<i8,N>{type Mask=Self;fn simd_eq(self,other:Self)->Self::Mask{loop{}}}
pub trait Select<T>{fn select(self,true_values:T,false_values:T)->T;}
impl<T,U,const N:usize> Select<Simd<T,N>> for Mask<U,N> where T:SimdElement,U:MaskElement{fn select(self,true_values:Simd<T,N>,false_values:Simd<T,N>)->Simd<T,N>{loop{}}}
impl<T,const N:usize> Select<Simd<T,N>> for u64 where T:SimdElement{fn select(self,true_values:Simd<T,N>,false_values:Simd<T,N>)->Simd<T,N>{loop{}}}
impl<T,U,const N:usize> Select<Mask<T,N>> for Mask<U,N> where T:MaskElement,U:MaskElement{fn select(self,true_values:Mask<T,N>,false_values:Mask<T,N>)->Mask<T,N>{loop{}}}
pub fn f<const N:usize>(a:Simd<i8,N>,b:Simd<i8,N>,m:Mask<i8,N>)->Simd<i8,N>{if b.simd_eq(Simd::splat(0 as _)).any(){a}else{b}}
