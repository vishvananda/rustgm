#![allow(dead_code,unused_imports,incomplete_features,internal_features,unused_macros)]
#![feature(min_specialization)]
trait Fast:Clone{}impl Fast for u8{}trait V:Clone{fn f()->u8;}impl<T:Clone>V for T{default fn f()->u8{1}}impl<T:Fast>V for T{fn f()->u8{2}}fn main(){}
