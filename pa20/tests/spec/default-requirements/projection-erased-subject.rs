#![allow(dead_code)]
trait P<X>{type O;}impl<X>P<X>for u8{type O=u16;}trait A{}struct S<T=u8>where <u8 as P<T>>::O:A{v:T}fn main(){}
