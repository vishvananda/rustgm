#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait A{fn a()->u8;}impl A for u8{fn a()->u8{7}}
trait Alias = where Self:A;
fn call<T:Alias>()->u8{T::a()}fn main(){let _=call::<u8>();}
