// Authored reducer following pinned core array/iter.rs and iter/iter_inner.rs receiver contracts.
#![allow(dead_code,unused_variables)]
trait PartialDrop{}
struct Inner<D:PartialDrop>{alive:usize,data:D}
impl<D:PartialDrop>Inner<D>{fn len(&self)->usize{self.alive}}
use core::ops::Deref;struct Hold<D>(D);impl<D>Deref for Hold<D>{type Target=D;fn deref(&self)->&D{&self.0}}
fn check<T,const N:usize>(x:&Hold<Inner<[T;N]>>)->usize{x.len()}
fn main(){}
