#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait A{fn a()->u8;}trait B{fn b()->u8;}
impl A for u8{fn a()->u8{3}}impl B for u8{fn b()->u8{4}}
trait Both=A+B;trait Again=Both;
fn sum<T:Again>()->u8{T::a()+T::b()}fn main(){println!("{}",sum::<u8>());}
