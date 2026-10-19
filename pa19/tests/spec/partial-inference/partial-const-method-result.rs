#![allow(dead_code)]
trait A<T> { fn value()->usize; }
impl<const N:usize> A<[u8;N]> for u8 { fn value()->usize { N } }
trait B {}
impl B for [u8;3] {}
fn need<T>() ->usize where u8:A<T>, T:B { <u8 as A<T>>::value() }
pub fn answer()->usize { need() }
fn main(){ let values=[0u8]; let _=values[if answer()==3 {0} else {1}]; }
