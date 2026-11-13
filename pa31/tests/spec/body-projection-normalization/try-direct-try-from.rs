#![feature(no_core,lang_items,rustc_attrs,repr_simd)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
// Authored reducer of portable-simd's vector.rs `TryFrom<&[T]>` for `Simd` (pinned core 88d9e12a):
// conversion, array and Result try impls follow core's convert/mod.rs, array/mod.rs and result.rs; bodies are reduced.
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl<T:Copy,const N:usize> Clone for [T;N]{fn clone(&self)->Self{*self}}impl<T:Copy,const N:usize> Copy for [T;N]{}
pub enum Infallible{}
pub enum Result<T,E>{Ok(T),Err(E)}
use crate::Result::{Ok,Err};
pub enum ControlFlow<B,C>{#[lang="Continue"]Continue(C),#[lang="Break"]Break(B)}
pub trait FromResidual<R=<Self as Try>::Residual>{#[lang="from_residual"]fn from_residual(residual:R)->Self;}
#[lang="Try"]pub trait Try:FromResidual{
    type Output;
    type Residual:Residual<Self::Output>;
    #[lang="from_output"]fn from_output(output:Self::Output)->Self;
    #[lang="branch"]fn branch(self)->ControlFlow<Self::Residual,Self::Output>;
}
pub trait Residual<O>:Sized{type TryType:Try<Output=O,Residual=Self>;}
#[lang="into_try_type"]pub fn residual_into_try_type<R:Residual<O>,O>(value:R)-><R as Residual<O>>::TryType{
    <<R as Residual<O>>::TryType as FromResidual<R>>::from_residual(value)
}
impl<T,E> Residual<T> for Result<Infallible,E>{type TryType=Result<T,E>;}
impl<T,E> Try for Result<T,E>{
    type Output=T;
    type Residual=Result<Infallible,E>;
    fn from_output(output:T)->Self{Ok(output)}
    fn branch(self)->ControlFlow<Result<Infallible,E>,T>{match self{Ok(v)=>ControlFlow::Continue(v),Err(e)=>ControlFlow::Break(Err(e))}}
}
impl<T,E,F:From<E>> FromResidual<Result<Infallible,E>> for Result<T,F>{fn from_residual(residual:Result<Infallible,E>)->Self{loop{}}}
pub trait From<T>:Sized{fn from(value:T)->Self;}
pub trait Into<T>:Sized{fn into(self)->T;}
impl<T,U> Into<U> for T where U:From<T>{fn into(self)->U{U::from(self)}}
impl<T> From<T> for T{fn from(t:T)->T{t}}
pub trait TryFrom<T>:Sized{type Error;fn try_from(value:T)->Result<Self,Self::Error>;}
pub trait TryInto<T>:Sized{type Error;fn try_into(self)->Result<T,Self::Error>;}
impl<T,U> TryInto<U> for T where U:TryFrom<T>{type Error=U::Error;fn try_into(self)->Result<U,U::Error>{U::try_from(self)}}
impl<T,U> TryFrom<U> for T where U:Into<T>{type Error=Infallible;fn try_from(value:U)->Result<Self,Self::Error>{Ok(U::into(value))}}
pub struct TryFromSliceError(());
impl<T,const N:usize> TryFrom<&[T]> for [T;N] where T:Copy{type Error=TryFromSliceError;fn try_from(slice:&[T])->Result<[T;N],TryFromSliceError>{loop{}}}
impl<T,const N:usize> TryFrom<&mut [T]> for [T;N] where T:Copy{type Error=TryFromSliceError;fn try_from(slice:&mut [T])->Result<[T;N],TryFromSliceError>{loop{}}}
impl<'a,T,const N:usize> TryFrom<&'a [T]> for &'a [T;N]{type Error=TryFromSliceError;fn try_from(slice:&'a [T])->Result<&'a [T;N],TryFromSliceError>{loop{}}}
impl<'a,T,const N:usize> TryFrom<&'a mut [T]> for &'a mut [T;N]{type Error=TryFromSliceError;fn try_from(slice:&'a mut [T])->Result<&'a mut [T;N],TryFromSliceError>{loop{}}}
mod vsealed{pub trait Sealed{}}
pub unsafe trait SimdElement:vsealed::Sealed+Copy{}
impl vsealed::Sealed for u8{}unsafe impl SimdElement for u8{}
#[repr(simd)]pub struct Simd<T,const N:usize>([T;N]) where T:SimdElement;
impl<T,const N:usize> Simd<T,N> where T:SimdElement{pub fn from_array(array:[T;N])->Self{loop{}}}
impl<T,const N:usize> TryFrom<&[T]> for Simd<T,N> where T:SimdElement{
    type Error=TryFromSliceError;
    fn try_from(slice:&[T])->Result<Self,TryFromSliceError>{
        Ok(Self::from_array(<[T;N]>::try_from(slice)?))
    }
}
