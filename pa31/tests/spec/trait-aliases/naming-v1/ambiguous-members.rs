#![feature(trait_alias)]
trait Left{type Item;}
trait Right{type Item;}
trait Both=Left+Right;
pub fn consumer<T:Both>(_:T::Item){}
