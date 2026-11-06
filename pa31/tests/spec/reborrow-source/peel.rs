#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared,PhantomData};
struct Source<'a,T>{value:&'a mut &'a mut T}
impl<'a,T>Reborrow for Source<'a,T>{}
struct Target<'a,T>{value:&'a T}
impl<T>Clone for Target<'_,T>{fn clone(&self)->Self{Self{value:self.value}}}
impl<T>Copy for Target<'_,T>{}
impl<'a,T>CoerceShared<Target<'a,T>>for Source<'a,T>{}
