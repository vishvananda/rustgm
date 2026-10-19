#![allow(dead_code)]
trait P{type O;}impl P for u8{type O=&'static u16;}trait A<X>{}struct S<T=u8>where T:A<<u8 as P>::O>{v:T}fn main(){}
