#![allow(dead_code)]
trait A {}
impl A for u8 {}
fn need<T:A>() {}
fn main(){need();}
