#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
trait T{fn f()->u8{9}} struct S;impl T for S{} fn main(){let _=<S as T>::f();}
