#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait P{fn value()->u8;}impl P for u8{fn value()->u8{7}}trait A=P;trait D0 = A+A;trait D1 = D0+D0;trait D2 = D1+D1;trait D3 = D2+D2;trait D4 = D3+D3;trait D5 = D4+D4;trait D6 = D5+D5;trait D7 = D6+D6;fn run<T:D7>()->u8{T::value()}fn main(){println!("{}",run::<u8>());}
