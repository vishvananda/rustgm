#![allow(dead_code)]
struct W<T>(T);
trait A<T> {}
impl A<W<u8>> for u8 {}
impl A<(u16,)> for u8 {}
fn need<T>() where u8:A<T> {}
fn main(){need();}
