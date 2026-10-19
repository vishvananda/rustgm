#![allow(dead_code)]
struct W<T>(T);
trait A<T> {}
trait B { fn value()->u32; }
impl B for W<u16> { fn value()->u32 { 23 } }
trait C<T> {}
impl<U> C<W<U>> for u16 {}
impl<T> A<T> for u8 where u16:C<T> {}
fn need<T>() ->u32 where u8:A<T>, T:B { T::value() }
pub fn answer()->u32 { need() }
fn main(){ let values=[0u8]; let _=values[if answer()==23 {0} else {1}]; }
