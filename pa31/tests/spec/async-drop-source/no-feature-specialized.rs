#![feature(no_core,lang_items)]
#![no_core]
#![allow(internal_features,incomplete_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="pin"]#[repr(transparent)]pub struct Pin<P>{pointer:P}
#[lang="deref"]pub trait Deref:PointeeSized{#[lang="deref_target"]type Target:PointeeSized;fn deref(&self)->&Self::Target;}
impl<T:PointeeSized>Deref for Pin<&mut T>{type Target=T;fn deref(&self)->&T{&*self.pointer}}
impl<T:PointeeSized>LegacyReceiver for Pin<&mut T>{}
#[lang="future_trait"]pub trait Future{#[lang="future_output"]type Output;}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="drop"]pub trait Drop{fn drop(&mut self);}
#[lang="async_drop"]pub trait Actual{fn drop(&mut self);}
struct Token<T>(T);impl<T> Drop for Token<T>{fn drop(&mut self){}}
impl Actual for Token<u8>{fn drop(&mut self){}}