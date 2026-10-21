#![no_implicit_prelude]
extern crate core;
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn less<T:core::cmp::PartialOrd>(a:T,b:T)->bool{a<b}fn main(){check(less(-2i16,3));}
