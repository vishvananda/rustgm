#![feature(trait_alias)]
trait P{type Out;}
trait A=P<Out=u8>+Copy;
trait B=A;
trait Family{type Item<T:B>:Clone;}
pub fn consumer<X:Family,T:P<Out=u8>>(_:X::Item<T>){}
