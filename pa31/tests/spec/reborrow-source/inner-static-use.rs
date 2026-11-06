#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared,PhantomData};
struct Source<'a,T>{value:&'a mut T}
impl<'a,T>Reborrow for Source<'a,T>{}
struct Target<'a,T:'static>{value:&'static T,marker:PhantomData<&'a ()>}
impl<T>Clone for Target<'_,T>{fn clone(&self)->Self{Self{value:self.value,marker:PhantomData}}}
impl<T>Copy for Target<'_,T>{}
impl<'a,T:'static>CoerceShared<Target<'a,T>>for Source<'a,T>{}
pub fn demand()->&'static u8{let mut value=7;let source=Source{value:&mut value};let target:Target<'_,u8>=source;target.value}
