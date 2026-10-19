#![allow(dead_code)]
trait A<'a>{}struct S<T=u8>where T:A<'static>{v:T}
fn force(_:S){}
fn main(){}
