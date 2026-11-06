#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared,PhantomData};
struct Multi<'a,T>{value:&'a mut u8,extra:T}
impl<'a,T:Copy>Reborrow for Multi<'a,T>{}
fn take<T:Copy>(_:Multi<'_,T>){}
pub fn repeat(value:&mut u8){let value=Multi{value,extra:7u8};take::<u8>(value);take::<u8>(value);}
