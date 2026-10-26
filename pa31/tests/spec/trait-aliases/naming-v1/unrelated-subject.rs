#![feature(trait_alias)]
trait Left{type One;}
trait Other{type Item;}
trait A<X>=Left where X:Other;
pub fn consumer<T:A<U>,U>(_:T::Item){}
