#![feature(trait_alias)]
trait Left{type One;}
trait Other{type Item;}
trait A=Left where Self:Other;
pub fn consumer<T:A>(_:T::Item){}
