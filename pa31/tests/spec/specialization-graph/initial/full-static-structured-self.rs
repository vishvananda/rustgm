#![allow(dead_code,unused_imports,incomplete_features,internal_features,unused_macros)]
#![feature(specialization)]
trait V{fn f()->u8;}impl<T>V for T{default fn f()->u8{1}}impl<T:'static>V for &'static T{fn f()->u8{2}}fn main(){}
