#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared,PhantomData};
struct Source<'a>{value:&'a u8}
impl<'a>Reborrow for Source<'a>{}
struct Target<'a>{value:&'static u8,marker:PhantomData<&'a ()>}
impl Clone for Target<'_>{fn clone(&self)->Self{Self{value:self.value,marker:PhantomData}}}
impl Copy for Target<'_>{}
impl<'a>CoerceShared<Target<'a>>for Source<'a>{}
