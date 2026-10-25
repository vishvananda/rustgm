#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(min_specialization)]
trait T{fn f()->u8;fn g()->u8;}impl<X>T for X{default fn f()->u8{1}fn g()->u8{0}}impl T for u8{fn f()->u8{2}fn g()->u8{3}}fn main(){}
