#![feature(trait_alias)]
trait Identity{type Out:?Sized;}
trait Other{type Item;}
trait A=Identity<Out=Self> where <Self as Identity>::Out:Other;
pub fn consumer<T:A>(_:T::Item){}
