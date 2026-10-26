#![feature(trait_alias)]
trait Leaf<'a>{type Out;}
trait A=for<'unused>Leaf<'static,Out=u8>;
pub fn consumer<T:A>(value:T::Out)->T::Out{value}
