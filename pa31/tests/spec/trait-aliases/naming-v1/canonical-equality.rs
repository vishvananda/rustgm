#![feature(trait_alias)]
trait Leaf<X>{type Out;}
trait A<X>=Leaf<X,Out=u8>;
trait Both<X>=A<X>+A<X>;
pub fn consumer<T:Both<u16>>(value:T::Out)->T::Out{value}
