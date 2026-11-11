// Authored reducer following pinned core array/iter.rs and iter/iter_inner.rs receiver contracts.
#![allow(dead_code,unused_variables)]
trait PartialDrop{}
impl<T,const N:usize>PartialDrop for[T;N]{}
struct Inner<D:?Sized+PartialDrop>{alive:usize,data:D}
impl<D:?Sized+PartialDrop>Inner<D>{fn len(&self)->usize{self.alive}}
use core::mem::ManuallyDrop;fn check<T,const N:usize>(x:&ManuallyDrop<Inner<[T;N]>>)->usize{x.len()}
fn main(){}
