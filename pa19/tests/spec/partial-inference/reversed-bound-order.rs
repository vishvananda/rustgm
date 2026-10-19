#![allow(dead_code)]
struct W<T>(T);
trait A<T> {}
trait B { fn value()->u32; }
impl B for W<u16> { fn value()->u32 { 23 } }
impl<U> A<W<U>> for u8 {}
pub fn answer()->u32 { need() }
fn need<T>() ->u32 where T:B, u8:A<T> { T::value() }
fn main(){ let values=[0u8]; let _=values[if answer()==23 {0} else {1}]; }
