#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared,PhantomData};
struct Multi<'a,T>{value:&'a mut u8,extra:T}
impl<'a,T:Copy>Reborrow for Multi<'a,T>{}
