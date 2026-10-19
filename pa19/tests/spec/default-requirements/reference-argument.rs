#![allow(dead_code)]
trait A<F>{}struct S<T=u8>where T:A<&'static u8>{v:T}
fn main(){}
