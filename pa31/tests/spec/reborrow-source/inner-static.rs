#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared,PhantomData};
struct Source<'a,T>{value:&'a mut T}
impl<'a,T>Reborrow for Source<'a,T>{}
struct Target<'a,T:'static>{value:&'static T,marker:PhantomData<&'a ()>}
impl<T>Clone for Target<'_,T>{fn clone(&self)->Self{Self{value:self.value,marker:PhantomData}}}
impl<T>Copy for Target<'_,T>{}
impl<'a,T:'static>CoerceShared<Target<'a,T>>for Source<'a,T>{}
