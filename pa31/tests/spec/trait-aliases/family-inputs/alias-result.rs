#![feature(trait_alias)]
trait P{type Out;}
trait A=P<Out=u8>+Copy;
trait B=A;
trait Family{type Item<T:B>:B;}
