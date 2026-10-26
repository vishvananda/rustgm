#![feature(trait_alias)]
trait Leaf<X>{type Out;}
trait A<X>=Leaf<X>;
pub fn consumer<T:A<u8>+A<u16>>(_:T::Out){}
