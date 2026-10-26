#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(specialization)]
trait T {fn f()->u8;}
default impl<X>T for X{fn f()->u8{1}} impl T for u8{} fn main(){let _=<u8 as T>::f();}
