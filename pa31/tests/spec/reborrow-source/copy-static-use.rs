#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared,PhantomData};
struct Source<'a>{value:&'static u8,marker:PhantomData<&'a ()>}
impl<'a>Reborrow for Source<'a>{}
#[derive(Clone,Copy)]struct Target<'a>{value:&'static u8,marker:PhantomData<&'a ()>}
impl<'a>CoerceShared<Target<'static>>for Source<'a>{}
fn consume(_:Target<'static>){}
pub fn demand(){static VALUE:u8=7;let source=Source{value:&VALUE,marker:PhantomData};consume(source);}
