// Authored reducer following pinned core array/iter.rs and iter/iter_inner.rs receiver contracts.
#![allow(dead_code,unused_variables)]
trait PartialDrop{}
impl<T,const N:usize>PartialDrop for[T;N]{}
struct Inner<D:?Sized+PartialDrop>{alive:usize,data:D}
impl<D:?Sized+PartialDrop>Inner<D>{fn len(&self)->usize{self.alive}}
use core::ops::Deref;
struct Hold<D:?Sized>(D);
impl<D:?Sized>Deref for Hold<D>{type Target=D;fn deref(&self)->&D{&self.0}}
fn check<T,const N:usize>(x:&Hold<Inner<[T;N]>>)->usize{x.0.len()}
fn main(){}
