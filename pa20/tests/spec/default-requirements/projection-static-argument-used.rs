#![allow(dead_code)]
trait P<'a>{type O;}impl<'a>P<'a>for u8{type O=u16;}trait A<X>{}struct S<T=u8>where T:A<<u8 as P<'static>>::O>{v:T}fn force(_:S){}fn main(){}
