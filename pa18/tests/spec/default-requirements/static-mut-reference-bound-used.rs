#![allow(dead_code)]
struct S<T:'static=u8>where &'static mut T:Copy{v:T}
fn force(_:S){}
fn main(){}
