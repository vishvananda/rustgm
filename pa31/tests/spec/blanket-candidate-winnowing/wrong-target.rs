#![feature(no_core,lang_items,rustc_attrs)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
// Authored reducer of core's slice/iter/macros.rs `Default` for `Iter`/`IterMut`;
// interfaces follow pinned core's iter/traits and slice/iter.rs, bodies are `loop {}`.
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
pub enum Option<T>{None,Some(T)}
pub trait Iterator{type Item;fn next(&mut self)->Option<Self::Item>;}
pub trait IntoIterator{type Item;type IntoIter:Iterator<Item=Self::Item>;fn into_iter(self)->Self::IntoIter;}
impl<I:Iterator> IntoIterator for I{type Item=I::Item;type IntoIter=I;fn into_iter(self)->I{self}}
pub trait Default:Sized{fn default()->Self;}
pub struct Iter<'a,T>{slice:&'a [T]}
pub struct IterMut<'a,T>{slice:&'a mut [T]}
impl<'a,T> Iterator for Iter<'a,T>{type Item=&'a T;fn next(&mut self)->Option<&'a T>{loop{}}}
impl<'a,T> Iterator for IterMut<'a,T>{type Item=&'a mut T;fn next(&mut self)->Option<&'a mut T>{loop{}}}
impl<'a,T> IntoIterator for &'a [T]{type Item=&'a T;type IntoIter=Iter<'a,T>;fn into_iter(self)->Iter<'a,T>{loop{}}}
impl<'a,T> IntoIterator for &'a mut [T]{type Item=&'a mut T;type IntoIter=IterMut<'a,T>;fn into_iter(self)->IterMut<'a,T>{loop{}}}
impl<'a,T,const N:usize> IntoIterator for &'a [T;N]{type Item=&'a T;type IntoIter=Iter<'a,T>;fn into_iter(self)->Iter<'a,T>{loop{}}}
impl<'a,T,const N:usize> IntoIterator for &'a mut [T;N]{type Item=&'a mut T;type IntoIter=IterMut<'a,T>;fn into_iter(self)->IterMut<'a,T>{loop{}}}
pub fn f<'a>() -> IterMut<'a, u8> { (&[]).into_iter() }
