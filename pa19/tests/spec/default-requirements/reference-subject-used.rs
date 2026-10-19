#![allow(dead_code)]
trait A{}struct S<T:'static=u8>where &'static T:A{v:T}
fn force(_:S){}
fn main(){}
