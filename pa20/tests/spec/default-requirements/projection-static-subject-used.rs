#![allow(dead_code)]
trait P<'a>{type O;}impl<'a>P<'a>for u8{type O=u16;}trait A{}struct S<T=u8>where T:P<'static>,<T as P<'static>>::O:A{v:T}fn force(_:S){}fn main(){}
