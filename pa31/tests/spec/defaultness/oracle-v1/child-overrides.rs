#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(min_specialization)]
trait T {fn f()->u8;}
impl<X> T for X {default fn f()->u8{1}}
impl T for u8{fn f()->u8{2}} fn main(){let _=(<u8 as T>::f(),<u16 as T>::f());}
