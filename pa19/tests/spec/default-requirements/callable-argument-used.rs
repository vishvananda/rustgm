#![allow(dead_code)]
trait A<F>{}struct S<T=u8>where T:A<for<'a>fn(&'a u8)>{v:T}
fn force(_:S){}
fn main(){}
