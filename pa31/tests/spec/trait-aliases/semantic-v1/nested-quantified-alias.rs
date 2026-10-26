#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait P<'a>{fn value()->u8;}impl<'a>P<'a> for u8{fn value()->u8{7}}trait Direct<'a>=P<'a>;trait All=for<'a>Direct<'a>;fn run<T:All>()->u8{<T as P<'static>>::value()}fn main(){println!("{}",run::<u8>());}
