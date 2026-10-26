#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(min_specialization)]
trait T{fn f()->u8{9} fn g()->u8;} impl<X>T for X{default fn f()->u8{1} default fn g()->u8{0}} impl T for u8{fn g()->u8{2}} fn main(){let _=(<u8 as T>::f(),<u8 as T>::g());}
