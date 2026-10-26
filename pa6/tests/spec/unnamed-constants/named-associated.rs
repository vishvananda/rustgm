#![allow(dead_code,unused_imports)]
trait T {const NAMED:u8;}struct S;impl T for S {const NAMED:u8=1;}impl S {const OTHER:u8=2;}fn main(){let _=<S as T>::NAMED;let _=S::OTHER;}
