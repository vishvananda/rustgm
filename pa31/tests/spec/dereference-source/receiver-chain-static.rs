#![feature(no_core,lang_items)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
pub mod ops {
 use crate::PointeeSized;
 #[lang="deref"]pub trait Deref:PointeeSized {
  #[lang="deref_target"]type Target:PointeeSized;
  fn deref(&self)->&Self::Target;
 }
 impl<T:PointeeSized>Deref for &T {type Target=T;fn deref(&self)->&T{*self}}
}
pub mod pin {
 use crate::{ops::Deref,LegacyReceiver};
 #[lang="pin"]pub struct Pin<P>{pub pointer:P}
 impl<P:Deref+'static>Deref for Pin<P>{
  type Target=P::Target;
  fn deref(&self)->&Self::Target{self.pointer.deref()}
 }
 impl<P:Deref+'static>LegacyReceiver for Pin<P>{}
}
type Layer0<'a,T>=&'a T;
type Layer1<'a,T>=pin::Pin<Layer0<'a,T>>;
type Layer2<'a,T>=pin::Pin<Layer1<'a,T>>;
type Layer3<'a,T>=pin::Pin<Layer2<'a,T>>;
type Layer4<'a,T>=pin::Pin<Layer3<'a,T>>;
type Layer5<'a,T>=pin::Pin<Layer4<'a,T>>;
type Layer6<'a,T>=pin::Pin<Layer5<'a,T>>;
type Layer7<'a,T>=pin::Pin<Layer6<'a,T>>;
type Layer8<'a,T>=pin::Pin<Layer7<'a,T>>;
type Layer9<'a,T>=pin::Pin<Layer8<'a,T>>;
type Layer10<'a,T>=pin::Pin<Layer9<'a,T>>;
type Layer11<'a,T>=pin::Pin<Layer10<'a,T>>;
type Layer12<'a,T>=pin::Pin<Layer11<'a,T>>;
type Layer13<'a,T>=pin::Pin<Layer12<'a,T>>;
type Layer14<'a,T>=pin::Pin<Layer13<'a,T>>;
type Layer15<'a,T>=pin::Pin<Layer14<'a,T>>;
type Layer16<'a,T>=pin::Pin<Layer15<'a,T>>;
impl<P:'static>pin::Pin<P>{pub fn deep(self:Layer16<'static,Self>){}}
