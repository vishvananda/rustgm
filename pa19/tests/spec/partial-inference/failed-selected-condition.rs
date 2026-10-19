#![allow(dead_code)]
struct W<T>(T);
trait A<T> {}
trait B { fn value()->u32; }
impl B for W<u16> { fn value()->u32 { 23 } }
trait Missing {}
impl<U:Missing> A<W<U>> for u8 {}
fn need<T>() ->u32 where u8:A<T>, T:B { T::value() }
pub fn answer()->u32 { need() }
fn main(){ let _=answer(); }
